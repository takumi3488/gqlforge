use async_graphql_value::ConstValue;

use super::eval_http::{
    EvalHttp, WorkerContext, execute_grpc_request_with_dl, execute_raw_grpc_request,
    execute_raw_request, execute_request_with_dl, parse_graphql_response, set_headers,
};
use super::model::{CacheKey, IO};
use super::{DynamicRequest, EvalContext, ResolverContextLike};
use crate::core::config::GraphQLOperationType;
use crate::core::config::S3Operation;
use crate::core::data_loader::DataLoader;
use crate::core::graphql::GraphqlDataLoader;
use crate::core::grpc;
use crate::core::grpc::data_loader::GrpcDataLoader;
use crate::core::http::DataLoaderRequest;
use crate::core::ir::Error;

pub async fn eval_io<Ctx>(io: &IO, ctx: &mut EvalContext<'_, Ctx>) -> Result<ConstValue, Error>
where
    Ctx: ResolverContextLike + Sync,
{
    // Note: Handled the case separately for performance reasons. It avoids cache
    // key generation when it's not required
    let dedupe = io.dedupe();

    if !dedupe || !ctx.is_query() {
        return eval_io_inner(io, ctx).await;
    }
    if let Some(key) = io.cache_key(ctx) {
        ctx.request_ctx
            .cache
            .dedupe(&key, || async {
                ctx.request_ctx
                    .dedupe_handler
                    .dedupe(&key, || eval_io_inner(io, ctx))
                    .await
            })
            .await
    } else {
        eval_io_inner(io, ctx).await
    }
}

async fn eval_io_inner<Ctx>(io: &IO, ctx: &mut EvalContext<'_, Ctx>) -> Result<ConstValue, Error>
where
    Ctx: ResolverContextLike + Sync,
{
    match io {
        IO::Http { req_template, dl_id, hook, .. } => {
            let event_worker = &ctx.request_ctx.runtime.cmd_worker;
            let js_worker = &ctx.request_ctx.runtime.worker;
            let eval_http = EvalHttp::new(ctx, req_template, dl_id);
            let request = eval_http.init_request()?;
            let response = match (&event_worker, js_worker, hook) {
                (Some(worker), Some(js_worker), Some(hook)) => {
                    let worker_ctx = WorkerContext::new(worker, js_worker, hook);
                    eval_http.execute_with_worker(request, worker_ctx).await?
                }
                _ => eval_http.execute(request).await?,
            };

            Ok(response.body)
        }
        IO::GraphQL { req_template, field_name, dl_id, .. } => {
            let req = req_template.to_request(ctx)?;
            let request = DynamicRequest::new(req);
            let res = if ctx.request_ctx.upstream.batch.is_some()
                && matches!(req_template.operation_type, GraphQLOperationType::Query)
            {
                let data_loader: Option<&DataLoader<DataLoaderRequest, GraphqlDataLoader>> =
                    dl_id.and_then(|dl| ctx.request_ctx.gql_data_loaders.get(dl.as_usize()));
                execute_request_with_dl(ctx, request, data_loader).await?
            } else {
                execute_raw_request(ctx, request).await?
            };

            set_headers(ctx, &res);
            parse_graphql_response(ctx, res, field_name)
        }
        IO::Grpc { req_template, dl_id, hook, .. } => {
            let rendered = req_template.render(ctx)?;
            let worker = &ctx.request_ctx.runtime.worker;

            let res = if ctx.request_ctx.upstream.batch.is_some() &&
                    // TODO: share check for operation_type for resolvers
                    matches!(req_template.operation_type, GraphQLOperationType::Query)
            {
                let data_loader: Option<&DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>> =
                    dl_id.and_then(|index| ctx.request_ctx.grpc_data_loaders.get(index.as_usize()));
                execute_grpc_request_with_dl(ctx, rendered, data_loader).await?
            } else {
                let req = rendered.to_request()?;
                execute_raw_grpc_request(ctx, req, &req_template.operation).await?
            };

            let res = match (worker.as_ref(), hook.as_ref()) {
                (Some(worker), Some(hook)) => hook.on_response(worker, res).await?,
                _ => res,
            };
            set_headers(ctx, &res);

            Ok(res.body)
        }
        IO::GrpcStream { .. } => {
            // GrpcStream is handled by the subscription layer, not eval_io
            Err(Error::IO(
                "GrpcStream should be resolved via subscription stream, not eval_io".to_string(),
            ))
        }
        IO::GraphQLStream { .. } => Err(Error::IO(
            "GraphQLStream should be resolved via subscription stream, not eval_io".to_string(),
        )),
        IO::HttpStream { .. } => Err(Error::IO(
            "HttpStream should be resolved via subscription stream, not eval_io".to_string(),
        )),
        IO::Postgres { req_template, dl_id: _, .. } => {
            let rendered = req_template
                .render(ctx)
                .map_err(|e| Error::IO(e.to_string()))?;
            let pg = ctx
                .request_ctx
                .runtime
                .postgres
                .as_ref()
                .ok_or_else(|| Error::IO("PostgreSQL runtime not configured".to_string()))?;
            let result = pg
                .execute(&rendered.sql, &rendered.params)
                .await
                .map_err(|e| Error::IO(e.to_string()))?;
            Ok(result)
        }
        IO::Js { name } => {
            match ctx
                .request_ctx
                .runtime
                .worker
                .as_ref()
                .zip(ctx.value().cloned())
            {
                Some((worker, value)) => {
                    let val = worker.call(name, value).await?;
                    Ok(val.unwrap_or_default())
                }
                _ => Ok(ConstValue::Null),
            }
        }
        IO::S3 { req_template, .. } => {
            let rendered = req_template.render(ctx);
            let link_id = rendered.link_id.as_deref();
            let s3 = ctx
                .request_ctx
                .runtime
                .s3
                .get(link_id.unwrap_or(""))
                .or_else(|| ctx.request_ctx.runtime.s3.values().next())
                .ok_or_else(|| Error::IO("S3 runtime not configured".to_string()))?;

            match rendered.operation {
                S3Operation::GetPresignedUrl => {
                    let key = rendered.key.as_deref().ok_or_else(|| {
                        Error::IO("S3 GET_PRESIGNED_URL requires a key".to_string())
                    })?;
                    let url = s3
                        .get_presigned_url(&rendered.bucket, key, rendered.expiration)
                        .await
                        .map_err(|e| Error::IO(e.to_string()))?;
                    Ok(ConstValue::String(url))
                }
                S3Operation::PutPresignedUrl => {
                    let key = rendered.key.as_deref().ok_or_else(|| {
                        Error::IO("S3 PUT_PRESIGNED_URL requires a key".to_string())
                    })?;
                    let url = s3
                        .put_presigned_url(
                            &rendered.bucket,
                            key,
                            rendered.expiration,
                            rendered.content_type.as_deref(),
                        )
                        .await
                        .map_err(|e| Error::IO(e.to_string()))?;
                    Ok(ConstValue::String(url))
                }
                S3Operation::List => {
                    let result = s3
                        .list_objects(&rendered.bucket, rendered.prefix.as_deref())
                        .await
                        .map_err(|e| Error::IO(e.to_string()))?;
                    Ok(result)
                }
                S3Operation::Delete => {
                    let key = rendered
                        .key
                        .as_deref()
                        .ok_or_else(|| Error::IO("S3 DELETE requires a key".to_string()))?;
                    let result = s3
                        .delete_object(&rendered.bucket, key)
                        .await
                        .map_err(|e| Error::IO(e.to_string()))?;
                    Ok(result)
                }
            }
        }
    }
}
