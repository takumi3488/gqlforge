import React from "react"
import Sidebar from "@theme-original/DocRoot/Layout/Sidebar"

const SidebarWrapper = (props: SidebarConfig) => {
  return (
    <div className="sidebar-search-container place-items-center flex flex-col lg:mb-[100px]">
      <Sidebar {...props} />
    </div>
  )
}

export default SidebarWrapper
