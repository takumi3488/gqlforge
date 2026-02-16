# Auth Protected with Expression

```yaml @config
server:
  port: 8000
links:
  - id: jwks
    src: jwks.json
    type: Jwks
```

```graphql @schema
schema {
  query: Query
}

type Query {
  adminDashboard: Dashboard @protected(expr: "claims.role == 'admin'") @expr(body: { stats: "admin stats" })
  publicData: String! @expr(body: "public")
  protectedBasic: String! @protected @expr(body: "protected data")
}

type Dashboard {
  stats: String!
}
```

```json @file:jwks.json
{
  "keys": [
    {
      "kty": "RSA",
      "use": "sig",
      "alg": "RS256",
      "kid": "I48qMJp566SSKQogYXYtHBo9q6ZcEKHixNPeNoxV1c8",
      "n": "ksMb5oMlhJ_HzAebCuBG6-v5Qc4J111ur7Aux6-8SbxzqFONsf2Bw6ATG8pAfNeZ-USA3_T1mGkYTDvfoggXnxsduWV_lePZKKOq_Qp_EDdzic1bVTJQDad3CXldR3wV6UFDtMx6cCLXxPZM5n76e7ybPt0iNgwoGpJE28emMZJXrnEUFzxwFMq61UlzWEumYqW3uOUVp7r5XAF5jQ_1nQAnpHBnRFzdNPVb3E6odMGu3jgp8mkPbPMP16Fund4LVplLz8yrsE9TdVrSdYJThylRWn_BwvJ0DjUcp8ibJya86iClUlixAmBwR9NdStHwQqHwmMXMKkTXo-ytRmSUobzxX9T8ESkij6iBhQpmDMD3FbkK30Y7pUVEBBOyDfNcWOhholjOj9CRrxu9to5rc2wvufe24VlbKb9wngS_uGfK4AYvVyrcjdYMFkdqw-Mft14HwzdO2BTS0TeMDZuLmYhj_bu5_g2Zu6PH5OpIXF6Fi8_679pCG8wWAcFQrFrM0eA70wD_SqD_BXn6pWRpFXlcRy_7PWTZ3QmC7ycQFR6Wc6Px44y1xDUoq3rH0RlZkeicfvP6FRlpjFU7xF6LjAfd9ciYBZfJll6PE7zf-i_ZXEslv-tJ5-30-I4Slwj0tDrZ2Z54OgAg07AIwAiI5o4y-0vmuhUscNpfZsGAGhE",
      "e": "AQAB"
    }
  ]
}
```

```yml @test
# Test 1: Access admin dashboard without token - should fail (auth)
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        adminDashboard {
          stats
        }
      }

# Test 2: Access admin dashboard with valid JWT (sub=you, no role claim) - should fail (expr)
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Bearer eyJhbGciOiJSUzI1NiIsImtpZCI6Ikk0OHFNSnA1NjZTU0tRb2dZWFl0SEJvOXE2WmNFS0hpeE5QZU5veFYxYzgifQ.eyJleHAiOjIwMTkwNTY0NDEuMCwiaXNzIjoibWUiLCJzdWIiOiJ5b3UiLCJhdWQiOlsidGhlbSJdfQ.cU-hJgVGWxK3-IBggYBChhf3FzibBKjuDLtq2urJ99FVXIGZls0VMXjyNW7yHhLLuif_9t2N5UIUIq-hwXVv7rrGRPCGrlqKU0jsUH251Spy7_ppG5_B2LsG3cBJcwkD4AVz8qjT3AaE_vYZ4WnH-CQ-F5Vm7wiYZgbdyU8xgKoH85KAxaCdJJlYOi8mApE9_zcdmTNJrTNd9sp7PX3lXSUu9AWlrZkyO-HhVbXFunVtfduDuTeVXxP8iw1wt6171CFbPmQJU_b3xCornzyFKmhSc36yvlDfoPPclWmWeyOfFEp9lVhQm0WhfDK7GiuRtaOxD-tOvpTjpcoZBeJb7bSg2OsneyeM_33a0WoPmjHw8WIxbroJz_PrfE72_TzbcTSDttKAv_e75PE48Vvx0661miFv4Gq8RBzMl2G3pQMEVCOm83v7BpodfN_YVJcqZJjVHMA70TZQ4K3L4_i9sIK9jJFfwEDVM7nsDnUu96n4vKs1fVvAuieCIPAJrfNOUMy7TwLvhnhUARsKnzmtNNrJuDhhBx-X93AHcG3micXgnqkFdKn6-ZUZ63I2KEdmjwKmLTRrv4n4eZKrRN-OrHPI4gLxJUhmyPAHzZrikMVBcDYfALqyki5SeKkwd4v0JAm87QzR4YwMdKErr0Xa5JrZqHGe2TZgVO4hIc-KrPw
  body:
    query: |
      query {
        adminDashboard {
          stats
        }
      }

# Test 3: Access admin dashboard with JWT containing role=admin - should succeed (expr passes)
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Bearer eyJhbGciOiJSUzI1NiIsImtpZCI6Ikk0OHFNSnA1NjZTU0tRb2dZWFl0SEJvOXE2WmNFS0hpeE5QZU5veFYxYzgiLCJ0eXAiOiJKV1QifQ.eyJleHAiOjIwMTkwNTY0NDEuMCwiaXNzIjoibWUiLCJzdWIiOiJ5b3UiLCJhdWQiOlsidGhlbSJdLCJyb2xlIjoiYWRtaW4ifQ.YueWLPyGFQWJ0FEwF3erUE1ue0cpdPSXzCpaQs_BHTmDord7LH4XqWTwZfaWsk2W6ErkYPQLZ6La5_3vBGk2xgwY68pieZC2wCdlrfy13cTby6By0GjuiFN7Fk65R-QGuTDwvFmmKaM60InIQOHG70PIR1z9zRjf_q1U6HkiyekZBjjWTCvBaPn7uD7scWZxdGjIqlV_Bt7dWh6fvj-xGwaH9JLU4QZWv6mYAA2E0uY9ZSWgYgC4nVs_vcQVNVAdv8cifyaidsyJnQVcD-SrCkVsrBnIj8R8UmTPj2eKyqcL4u9TX2mkYoOw8jdK563eoT7Y8nd6BPqDFpQRBrsuM1y5fmTlYxif7ZBWHaMARyI6DiDlfvHZCBnaevAdt_HzyX4leq8iY4sVjNI2f8LmpA5or-A-1yVRF6r4-Ca9D0yRJ9xDk9_KaKPptAJyacwmbeSdLceho8exkNfBZqau_F5Mtm3qeRkkJtB3M-S_hPoDgwEP2tx8ZaSTncXPtV4NCfjk2X5Iq0IZsiKbqE_V893LfMn__TuHJyTNZWtVzhK8tg0sXL-La0TDqux5JCFWSrR8ESa6QggKiB59L7sZTvePj6N7m5qGCb-j8B8MTnvme0iKBCcIVLzhBnRdxkZrYUbIiiWFnH-zarkjL4rJb9yznRWwv0LoNSTHIZIXm10
  body:
    query: |
      query {
        adminDashboard {
          stats
        }
      }

# Test 4: Basic @protected without expr still works with valid token
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Bearer eyJhbGciOiJSUzI1NiIsImtpZCI6Ikk0OHFNSnA1NjZTU0tRb2dZWFl0SEJvOXE2WmNFS0hpeE5QZU5veFYxYzgifQ.eyJleHAiOjIwMTkwNTY0NDEuMCwiaXNzIjoibWUiLCJzdWIiOiJ5b3UiLCJhdWQiOlsidGhlbSJdfQ.cU-hJgVGWxK3-IBggYBChhf3FzibBKjuDLtq2urJ99FVXIGZls0VMXjyNW7yHhLLuif_9t2N5UIUIq-hwXVv7rrGRPCGrlqKU0jsUH251Spy7_ppG5_B2LsG3cBJcwkD4AVz8qjT3AaE_vYZ4WnH-CQ-F5Vm7wiYZgbdyU8xgKoH85KAxaCdJJlYOi8mApE9_zcdmTNJrTNd9sp7PX3lXSUu9AWlrZkyO-HhVbXFunVtfduDuTeVXxP8iw1wt6171CFbPmQJU_b3xCornzyFKmhSc36yvlDfoPPclWmWeyOfFEp9lVhQm0WhfDK7GiuRtaOxD-tOvpTjpcoZBeJb7bSg2OsneyeM_33a0WoPmjHw8WIxbroJz_PrfE72_TzbcTSDttKAv_e75PE48Vvx0661miFv4Gq8RBzMl2G3pQMEVCOm83v7BpodfN_YVJcqZJjVHMA70TZQ4K3L4_i9sIK9jJFfwEDVM7nsDnUu96n4vKs1fVvAuieCIPAJrfNOUMy7TwLvhnhUARsKnzmtNNrJuDhhBx-X93AHcG3micXgnqkFdKn6-ZUZ63I2KEdmjwKmLTRrv4n4eZKrRN-OrHPI4gLxJUhmyPAHzZrikMVBcDYfALqyki5SeKkwd4v0JAm87QzR4YwMdKErr0Xa5JrZqHGe2TZgVO4hIc-KrPw
  body:
    query: |
      query {
        protectedBasic
      }

# Test 5: Public data accessible without any token
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        publicData
      }
```
