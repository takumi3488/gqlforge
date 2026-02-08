import React, {ReactNode} from "react"
import {composeProviders} from "@docusaurus/theme-common"
import {
  ColorModeProvider,
  AnnouncementBarProvider,
  ScrollControllerProvider,
  NavbarProvider,
  PluginHtmlClassNameProvider,
} from "@docusaurus/theme-common/internal"
import {DocsPreferredVersionContextProvider} from "@docusaurus/plugin-content-docs/client"
import Footer from "@site/src/components/shared/Footer"

type LayoutProviderProps = {
  children: ReactNode
}

const Provider = composeProviders([
  ColorModeProvider,
  AnnouncementBarProvider,
  ScrollControllerProvider,
  DocsPreferredVersionContextProvider,
  PluginHtmlClassNameProvider,
  NavbarProvider,
])

const LayoutProvider = ({children}: LayoutProviderProps) => {
  return (
    <Provider>
      {children}
      <Footer />
    </Provider>
  )
}

export default LayoutProvider
