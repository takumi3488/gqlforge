type Feature = {
  title: string
  description: string
  icon: string
}

type Social = {
  id: number
  name: string
  image?: React.FunctionComponent<React.SVGProps<SVGSVGElement>>
  href: string
}

type FooterLink = {
  name: string
  link: string
}

type FooterItem = {
  title: string
  items: FooterLink[]
}

type SidebarLink = {
  type: "link"
  label: string
  href: string
  docId: string
  unlisted: boolean
}

type SidebarCategory = {
  type: "category"
  label: string
  collapsible: boolean
  collapsed: boolean
  items: SidebarLink[]
}

type SidebarItem = {
  type: "category" | "link"
  label: string
  collapsible?: boolean
  collapsed?: boolean
  items?: SidebarLink[]
  href?: string
  docId?: string
  unlisted?: boolean
}

type SidebarConfig = {
  sidebar: SidebarItem[]
  hiddenSidebarContainer: boolean
}

declare module "react-platform-js"
