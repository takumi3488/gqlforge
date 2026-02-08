import Link from "@docusaurus/Link"
import clsx from "clsx"
import React from "react"

type LinkButtonProps = {
  title: string
  href?: string
  variant?: "primary" | "secondary" | "outline"
  className?: string
}

const LinkButton = ({title, href, variant = "primary", className}: LinkButtonProps): JSX.Element => {
  const variants = {
    primary: "bg-gqlForge-dark-500 text-white hover:text-white",
    secondary: "bg-gqlForge-yellow text-gqlForge-dark-500 hover:text-gqlForge-dark-500",
    outline: "border-2 border-gqlForge-dark-500 text-gqlForge-dark-500 hover:text-gqlForge-dark-500",
  }

  return (
    <Link
      to={href}
      className={clsx(
        "flex items-center justify-center hover:no-underline font-bold px-8 py-3 rounded-xl text-title-small h-12 sm:h-16",
        variants[variant],
        className,
      )}
    >
      {title}
    </Link>
  )
}

export default LinkButton
