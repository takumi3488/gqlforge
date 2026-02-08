import React from "react"

type SectionTitleProps = {
  title: string
}

const SectionTitle = ({title}: SectionTitleProps): JSX.Element => {
  return (
    <div className="text-content-tiny sm:text-title-tiny text-gqlForge-light-600 font-space-mono">
      <span>&gt;_ {title}</span>
    </div>
  )
}

export default SectionTitle
