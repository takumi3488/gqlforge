import {themes as prismThemes} from "prism-react-renderer"
import type * as Preset from "@docusaurus/preset-classic"
import prismTheme from "./src/theme/CodeBlock/theme"
import type {Config} from "@docusaurus/types"

const title = "GQLForge"
const organization = "gqlforge"
const project = "gqlforge"

export default {
  title,
  trailingSlash: true,
  tagline: "GraphQL platform engineered for scale",
  headTags: [
    {
      tagName: "script",
      attributes: {
        type: "application/ld+json",
      },
      innerHTML: JSON.stringify({
        "@context": "https://schema.org/",
        "@type": "WebSite",
        name: "GQLForge",
        url: "https://gqlforge.pages.dev/",
      }),
    },
  ],
  url: "https://gqlforge.pages.dev",
  baseUrl: "/",
  onBrokenLinks: "throw",
  onBrokenMarkdownLinks: "throw",
  onBrokenAnchors: "throw",
  favicon: "images/favicon.ico",

  organizationName: organization,
  projectName: project,

  i18n: {
    defaultLocale: "en",
    locales: ["en"],
    localeConfigs: {
      en: {
        label: "English",
      },
    },
  },
  future: {
    experimental_faster: false,
  },
  presets: [
    [
      "classic",
      {
        docs: {
          sidebarPath: require.resolve("./sidebars.ts"),
          showLastUpdateTime: true,
          sidebarCollapsible: true,
          editUrl: `https://github.com/${organization}/${project}/tree/main/docs`,
        },
        blog: false,
        theme: {
          customCss: require.resolve("./src/css/custom.css"),
        },
        sitemap: {
          changefreq: "weekly",
          priority: 0.5,
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: "icons/companies/gqlforge.svg",
    navbar: {
      hideOnScroll: true,
      logo: {
        alt: "GQLForge",
        src: "icons/companies/gqlforge.svg",
      },
      items: [
        {to: "/", label: "Home", position: "left", activeBaseRegex: "^/$"},
        {to: "/docs/getting-started", label: "Docs", position: "left"},
        {to: "/playground", label: "Playground", position: "left"},
        {
          href: `https://github.com/${organization}/${project}`,
          label: "GitHub",
          position: "right",
        },
      ],
    },
    prism: {
      theme: prismTheme,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ["protobuf", "json", "diff"],
    },
    colorMode: {
      disableSwitch: true,
      defaultMode: "light",
      respectPrefersColorScheme: false,
    },
    tableOfContents: {},
  } satisfies Preset.ThemeConfig,
  plugins: [
    async function tailwindPlugin() {
      return {
        name: "docusaurus-tailwindcss",
        configurePostCss(postcssOptions) {
          return {
            ...postcssOptions,
            plugins: [...postcssOptions.plugins, require("tailwindcss"), require("autoprefixer")],
          }
        },
      }
    },
  ],
} satisfies Config
