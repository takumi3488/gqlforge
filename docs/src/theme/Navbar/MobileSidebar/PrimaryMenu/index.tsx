import React from "react"
import {useThemeConfig} from "@docusaurus/theme-common"
import {useNavbarMobileSidebar} from "@docusaurus/theme-common/internal"
import NavbarItem, {type Props as NavbarItemConfig} from "@theme/NavbarItem"

const useNavbarItems = () => {
  return useThemeConfig().navbar.items as NavbarItemConfig[]
}

const NavbarMobilePrimaryMenu = (): JSX.Element => {
  const mobileSidebar = useNavbarMobileSidebar()
  const items = useNavbarItems()

  return (
    <ul className="menu__list">
      {items.map((item, i) => (
        <NavbarItem mobile {...item} onClick={() => mobileSidebar.toggle()} key={i} />
      ))}
    </ul>
  )
}

export default NavbarMobilePrimaryMenu
