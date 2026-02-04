# Astro Parser Report

**Parser**: oxc `parseAstroSync` from `napi/parser`  
**Source**: `../docs` (79 Astro files)  
**Date**: 2026-02-04

## Summary

| Status    | Count  | Percentage |
| --------- | ------ | ---------- |
| Passed    | 79     | 100%       |
| Failed    | 0      | 0%         |
| **Total** | **79** | 100%       |

All 79 Astro files from the `../docs` directory parse successfully.

---

## Passed Files (79)

- `src/components/BackendGuidesNav.astro`
- `src/components/Badge.astro`
- `src/components/BrandLogo.astro`
- `src/components/Button.astro`
- `src/components/CMSGuidesNav.astro`
- `src/components/Checklist.astro`
- `src/components/ContributorList.astro`
- `src/components/DeployGuidesNav.astro`
- `src/components/DontEditWarning.astro`
- `src/components/FacePile.astro`
- `src/components/FluidGrid.astro`
- `src/components/FooterLinks.astro`
- `src/components/IntegrationsNav.astro`
- `src/components/IslandsDiagram.astro`
- `src/components/Landing/Card.astro`
- `src/components/Landing/Discord.astro`
- `src/components/Landing/ListCard.astro`
- `src/components/Landing/SplitCard.astro`
- `src/components/LeftSidebar/Sponsors.astro`
- `src/components/LoopingVideo.astro`
- `src/components/MediaGuidesNav.astro`
- `src/components/MigrationGuidesNav.astro`
- `src/components/NavGrid/Card.astro`
- `src/components/NavGrid/CardsNav.astro`
- `src/components/NavGrid/Grid.astro`
- `src/components/NotFound.astro`
- `src/components/ReadMore.astro`
- `src/components/RecipeLinks.astro`
- `src/components/RecipesNav.astro`
- `src/components/RightSidebar/LearnAstroAd.astro`
- `src/components/RightSidebar/RandomizedAd.astro`
- `src/components/RightSidebar/ScrimbaAd.astro`
- `src/components/RightSidebar/StarlightBanner.astro`
- `src/components/ShowcaseCard.astro`
- `src/components/Since.astro`
- `src/components/SourcePR.astro`
- `src/components/Spoiler.astro`
- `src/components/TabGroup/InstallGuideTabGroup.astro`
- `src/components/Version.astro`
- `src/components/starlight/EditLink.astro`
- `src/components/starlight/Footer.astro`
- `src/components/starlight/Hero.astro`
- `src/components/starlight/Hero/FacePile.astro`
- `src/components/starlight/MarkdownContent.astro`
- `src/components/starlight/MobileMenuFooter.astro`
- `src/components/starlight/MobileTableOfContents.astro`
- `src/components/starlight/PageSidebar.astro`
- `src/components/starlight/PageTitle.astro`
- `src/components/starlight/Search.astro`
- `src/components/starlight/Sidebar.astro`
- `src/components/starlight/SiteTitle.astro`
- `src/components/starlight/TableOfContents.astro`
- `src/components/tabs/AstroJSXTabs.astro`
- `src/components/tabs/AstroVueTabs.astro`
- `src/components/tabs/JavascriptFlavorTabs.astro`
- `src/components/tabs/PackageManagerTabs.astro`
- `src/components/tabs/StaticSsrTabs.astro`
- `src/components/tabs/TabListItem.astro`
- `src/components/tabs/TabPanel.astro`
- `src/components/tabs/TabbedContent.astro`
- `src/components/tabs/UIFrameworkTabs.astro`
- `src/components/tutorial/Blanks.astro`
- `src/components/tutorial/Box.astro`
- `src/components/tutorial/CompletionConfetti.astro`
- `src/components/tutorial/Lede.astro`
- `src/components/tutorial/MobileTutorialNav.astro`
- `src/components/tutorial/MultipleChoice.astro`
- `src/components/tutorial/Option.astro`
- `src/components/tutorial/PreCheck.astro`
- `src/components/tutorial/Progress.astro`
- `src/components/tutorial/TutorialNav.astro`
- `src/components/tutorial/UnitProgressIcon.astro`
- `src/pages/404.astro`
- `src/pages/[...enRedirectSlug].astro`
- `src/pages/[lang]/404.astro`
- `src/pages/[lang]/index.astro`
- `src/pages/[lang]/install.astro`
- `src/pages/[lang]/tutorial.astro`
- `src/pages/index.astro`

---

## Fix Applied

A bug was found and fixed in `crates/oxc_parser/src/jsx/mod.rs` related to parsing `<style>` and `<script>` elements (raw text elements).

**Issue**: The parser was calling `next_jsx_child()` after parsing the opening tag's `>`, which would attempt to lex the raw content (CSS/JS) as JSX. This caused CSS selectors containing `>` (like `.foo > .bar`) to fail with "Unexpected token" errors.

**Fix**: Modified `parse_jsx_opening_element` to detect raw text elements (`<style>`, `<script>`) and skip the `next_jsx_child()` call for those elements. The `skip_raw_text_element_content` function now properly positions the lexer past the raw content without attempting to parse it as JSX.
