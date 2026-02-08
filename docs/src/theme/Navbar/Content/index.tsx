import React, {type ReactNode} from "react"
import {useThemeConfig, ErrorCauseBoundary} from "@docusaurus/theme-common"
import {splitNavbarItems, useNavbarMobileSidebar} from "@docusaurus/theme-common/internal"
import NavbarItem, {type Props as NavbarItemConfig} from "@theme/NavbarItem"
import NavbarColorModeToggle from "@theme/Navbar/ColorModeToggle"
import NavbarMobileSidebarToggle from "@theme/Navbar/MobileSidebar/Toggle"
import NavbarLogo from "@theme/Navbar/Logo"
import styles from "./styles.module.css"

const useNavbarItems = () => {
  return useThemeConfig().navbar.items as NavbarItemConfig[]
}

const NavbarItems = ({items}: {items: NavbarItemConfig[]}): JSX.Element => {
  return (
    <>
      {items.map((item, i) => (
        <ErrorCauseBoundary
          key={i}
          onError={(error) =>
            new Error(
              `A theme navbar item failed to render.
Please double-check the following navbar item (themeConfig.navbar.items) of your Docusaurus config:
${JSON.stringify(item, null, 2)}`,
            )
          }
        >
          <NavbarItem {...item} />
        </ErrorCauseBoundary>
      ))}
    </>
  )
}

const NavbarContentLayout = ({left, right}: {left: ReactNode; right: ReactNode}) => {
  return (
    <div className="navbar__inner">
      <div className="navbar__items">{left}</div>
      <div className="navbar__items navbar__items--right">{right}</div>
    </div>
  )
}

const NavbarContent = (): JSX.Element => {
  const mobileSidebar = useNavbarMobileSidebar()
  const items = useNavbarItems()
  const [leftItems, rightItems] = splitNavbarItems(items)

  return (
    <NavbarContentLayout
      left={
        <>
          {!mobileSidebar.disabled && <NavbarMobileSidebarToggle />}
          <NavbarLogo />
          <NavbarItems items={leftItems} />
        </>
      }
      right={
        <>
          <NavbarItems items={rightItems} />
          <NavbarColorModeToggle className={styles.colorModeToggle} />
        </>
      }
    />
  )
}

export default NavbarContent
