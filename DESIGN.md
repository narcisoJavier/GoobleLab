---
name: GoobleLab
description: A calm browser notebook studio with forest actions, cool surfaces and editorial headings.
colors:
  accent: "#235d43"
  ground: "#f4f7f5"
  surface: "#fff"
  summary: "#e6eee9"
  selected: "#edf7f0"
  ink: "#17251f"
  muted: "#58645d"
  border: "#cbd5ce"
  error: "#a52d29"
  hover: "#e5eee8"
  primary-hover: "#194b34"
  plan-ink: "#4f6557"
  badge-free: "#e9eeeb"
  badge-free-ink: "#425449"
  badge-pro: "#dcebe1"
  badge-pro-ink: "#28543e"
  workflow-disabled: "#f0f3f1"
  disabled-button: "#e2e8e3"
  disabled-button-ink: "#67736a"
  disabled-border: "#d1dbd4"
  disabled-field: "#eaf0ec"
  disabled-field-ink: "#647268"
typography:
  display:
    fontFamily: "Instrument Serif, Georgia, serif"
    fontSize: "clamp(46px, 4.6vw, 68px)"
    fontWeight: 400
    lineHeight: 1.02
    letterSpacing: "-0.025em"
  page-display:
    fontFamily: "Instrument Serif, Georgia, serif"
    fontSize: "clamp(42px, 5vw, 64px)"
    fontWeight: 400
    lineHeight: 1.07
    letterSpacing: "-0.025em"
  title:
    fontFamily: "Manrope, Segoe UI, sans-serif"
    fontSize: "26px"
    fontWeight: 650
    lineHeight: 1.25
    letterSpacing: "-0.035em"
  body:
    fontFamily: "Manrope, Segoe UI, sans-serif"
    fontSize: "15px"
    lineHeight: 1.6
  intro:
    fontFamily: "Manrope, Segoe UI, sans-serif"
    fontSize: "14px"
    lineHeight: 1.7
  button:
    fontFamily: "Manrope, Segoe UI, sans-serif"
    fontSize: "13px"
    fontWeight: 700
    lineHeight: 1.5
  workflow-title:
    fontFamily: "Manrope, Segoe UI, sans-serif"
    fontSize: "16px"
    fontWeight: 750
    letterSpacing: "-0.025em"
  workflow-description:
    fontFamily: "Manrope, Segoe UI, sans-serif"
    fontSize: "13px"
    lineHeight: 1.8
  label:
    fontFamily: "Manrope, Segoe UI, sans-serif"
    fontSize: "12px"
    fontWeight: 750
  badge:
    fontFamily: "Manrope, Segoe UI, sans-serif"
    fontSize: "11px"
    fontWeight: 700
  plan-label:
    fontFamily: "Manrope, Segoe UI, sans-serif"
    fontSize: "11px"
  plan-value:
    fontFamily: "Manrope, Segoe UI, sans-serif"
    fontSize: "12px"
    fontWeight: 650
rounded:
  control: "8px"
  notice: "12px"
  workflow: "14px"
  panel: "16px"
  badge: "20px"
  account-action: "24px"
  circle: "50%"
spacing:
  tight: "8px"
  item: "12px"
  field-gap: "18px"
  section: "28px"
  desktop-gutter: "32px"
  column-gap: "48px"
components:
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.surface}"
    typography: "{typography.button}"
    rounded: "{rounded.control}"
    padding: "10px 20px"
  button-primary-hover:
    backgroundColor: "{colors.primary-hover}"
  button-secondary:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    typography: "{typography.button}"
    rounded: "{rounded.control}"
    padding: "10px 20px"
  button-secondary-hover:
    backgroundColor: "{colors.hover}"
  field:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    padding: "12px 14px"
  workflow:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.workflow}"
    padding: "18px 16px 22px"
  workflow-selected:
    backgroundColor: "{colors.selected}"
  notebook-plan:
    backgroundColor: "{colors.summary}"
    textColor: "{colors.ink}"
    rounded: "{rounded.panel}"
    padding: "27px 23px"
  badge-free:
    backgroundColor: "{colors.badge-free}"
    textColor: "{colors.badge-free-ink}"
    typography: "{typography.badge}"
    rounded: "{rounded.badge}"
    padding: "3px 9px"
  badge-pro:
    backgroundColor: "{colors.badge-pro}"
    textColor: "{colors.badge-pro-ink}"
    typography: "{typography.badge}"
    rounded: "{rounded.badge}"
    padding: "3px 9px"
  progress-number-current:
    backgroundColor: "{colors.ink}"
    textColor: "{colors.surface}"
    rounded: "{rounded.circle}"
    size: "25px"
---

# Design System: GoobleLab

## Overview

**Creative North Star: "Open notebook studio"**

A quiet, spacious browser studio frames a practical notebook-building task. Forest green actions and a pale mint notebook plan sit on a cool near-white ground. Editorial serif headings establish a welcoming first impression; geometric sans-serif controls keep the work precise and easy to scan.

This is the implemented replacement browser world in `web/style.css`, `web/index.html` and `web/app.mjs`. The user rejected the former yellow browser direction and supplied [HackerRank](https://www.hackerrank.com/) and [Eloqwnt](https://www.eloqwnt.com/) as references. HackerRank informs the large serif/sans contrast, calm spacing and green/white relationship. Eloqwnt informs the airy geometric sans-serif typography, restrained navigation and softly rounded content surfaces. These are craft references; the application keeps its own product facts and workflow.

**Key Characteristics:**

- Cool open ground, white workflow choices and a mint notebook plan.
- Self-hosted Instrument Serif display typography and Manrope controls.
- A horizontal stepper, geometric workflow illustrations and restrained green states.
- Flat rounded surfaces, visible radio controls and clear keyboard focus.

The frontmatter and the eight sections below describe the current browser. The separate native legacy appendix preserves the optional Windows companion's earlier system and logical-pixel measurements; it does not constrain new browser screens. Browser measurements are CSS pixels. No shipping raster artwork is used: branding and workflow illustrations are SVG, and selected image previews are user-provided media.

## Colors

Forest green provides action and selection emphasis; cool neutrals and muted mint organize the work without a colored navigation rail.

### Primary

- **Forest green** (`accent`): primary actions, links, workflow selection borders, checked radios and keyboard focus.
- **Deep forest hover** (`primary-hover`): primary button hover. The header preview action instead begins in ink and turns forest on hover.

### Neutral

- **Cool ground** (`ground`) and **white surface** (`surface`): the page and interactive workflow surfaces.
- **Mint plan** (`summary`): notebook summary and the Pro comparison offering.
- **Selection mint** (`selected`): checked workflow fill, paired with its visible radio and forest border.
- **Green-black ink** (`ink`): headings, values, current step number and header preview action.
- **Sage gray** (`muted`) and **plan green-gray** (`plan-ink`): supporting copy and plan labels/help.
- **Soft outline** (`border`) and **hover mint** (`hover`): field/workflow separation and secondary action hover.
- **Free / Pro badge fills and inks** (`badge-free`, `badge-free-ink`, `badge-pro`, `badge-pro-ink`): small written plan labels with restrained tonal differences.
- **Disabled surface and text tokens**: distinguish unavailable workflow, button and field states while retaining readable labels.

**Error red** (`error`) is reserved for validation failures and error toasts.

**The Written State Rule.** Color supports a visible radio, a current-step number, text or a disabled control; it never carries plan availability or an error by itself.

## Typography

**Display font:** Instrument Serif (Georgia, serif fallback), regular only. **Interface font:** Manrope (Segoe UI, sans-serif fallback), variable weights 200–800. Both fonts are self-hosted under `web/fonts/` with their OFL license files and use `font-display: swap`.

The serif is used for the builder welcome, full-page headings and comparison status/price. All task headings, descriptions, editable values, labels and actions use Manrope. The browser no longer uses Georgia for routine guidance.

- **Display:** responsive welcome heading, with the second line in forest green. It appears on Template only; Settings and Export hide the welcome block to bring controls upward. At the mobile and small breakpoints its sizes become 51px and 47px.
- **Page display:** Saved projects and Free & Pro headings, reduced to 45px at the mobile breakpoint.
- **Title:** compact workspace heading, reduced to 24px on mobile.
- **Body / intro:** the document base and focused supporting paragraphs. Introductions stay within 70ch; quiet explanatory text within 75ch.
- **Workflow title / description:** bold sans-serif title above readable supporting copy. Wide-screen titles increase to 18px. Descriptions are 13px by default and 12px at compact/mobile sizes.
- **Button / label / badge:** compact, weighted interface labels, without general uppercase treatment.
- **Plan label / value:** 11px labels and 12px semibold values, with tabular numbers and wrapping for long filenames.

**The Task Type Rule.** Keep editorial scale in welcome and page headings; use the compact Manrope hierarchy for configuration and handoff.

## Layout

The desktop browser uses a centered container (1440px maximum) with 44px top, 32px horizontal and 56px bottom padding. A welcome row and bordered horizontal stepper span the container. Below, the flexible workspace and 286px notebook plan form two columns with a 48px gap. Settings and Export omit the welcome row. The browser uses normal document scrolling, with no independent native pane scrollers or sticky plan.

Workflow choices form three equal columns with 12px gaps at the default desktop size. Settings pair compatible fields in two columns. The action row wraps below a fine divider. Secondary pages use an 1180px maximum container, a horizontally scrollable project table, and two comparison offerings.

- **1500px minimum:** column gap increases to 64px and plan width to 320px. Workflow padding becomes 22px; plan padding becomes 30px.
- **1100px maximum:** the plan narrows to 250px and the column gap to 28px. Container padding becomes 35px 26px 45px. Workflow choices become one vertical list of horizontal rows, with illustration, copy and radio/badge columns. The stepper's extra note is hidden.
- **760px maximum:** navigation wraps into a second header row; the builder becomes one column with 30px 22px 40px padding. The plan follows the workspace, with two-column facts and actions. Step connectors disappear, workflow rows stay horizontal, page headings/actions stack, and comparison offerings become one column.
- **450px maximum:** horizontal gutters reduce to 18px; paired fields and plan actions become one column. Workflow illustration/copy/control columns tighten, input text increases to 16px, image selection stacks, and the small account plan badge is hidden while the preview action stays visible.

Use these actual CSS breakpoints. A screenshot's physical dimensions do not determine its browser viewport breakpoint.

## Elevation & Depth

Surfaces stay flat. Mint/white fills, fine borders and open space separate the notebook plan, workflow choices, fields and comparison offerings. The only shipping shadow is the fixed status toast (`0 8px 28px #17251f26`), with ink fill for ordinary status and error red for failures.

When `prefers-reduced-motion: no-preference` matches, buttons and workflow choices transition background/border colors over 150ms with `ease`. Toast entry combines an 8px upward translation and clip reveal over 180ms using `cubic-bezier(.16,1,.3,1)`. The reduced-motion path has no added entry animation or color transitions.

**The Flat Surface Rule.** Use tonal layering and border state for working surfaces; keep elevation for transient feedback.

## Shapes

The form language is softly rounded and geometrically simple: 8px controls, 14px workflow choices, 16px plan/comparison surfaces, 20px plan badges and a 24px header preview action. Step numbers are circular. Workflow selection changes its one-pixel border color and fill without changing thickness or occupied size.

Inline SVG workflow illustrations use thin, rounded geometric strokes. The notebook brand mark combines a dark rounded square and a pale mint line icon. These images identify workflow and product; they do not replace labels. Keyboard-visible focus uses a 3px forest outline with a 4px offset, including around a focused workflow label.

## Components

### Buttons

Primary actions use forest green, white text and a small CSS chevron. Secondary actions use a white fill, ink and a soft outline. Both have an 8px radius, 10px 20px padding and a 44px minimum height. Small actions are at least 40px high. The header preview action is a separate ink-filled pill. Text actions remain underlined and at least 44px high. Disabled buttons use explicit muted fill, text and border colors; busy Drive work disables the relevant controls.

### Inputs / Fields

Labels sit above full-width inputs, selects and textareas with an 8px radius, soft border, white fill, 12px 14px padding and a 46px minimum height. Prompt textareas begin at 185px and resize vertically. Hover strengthens the outline; focus uses the shared visible ring. Disabled generation controls remain visible with a muted fill and written Free/Pro explanation. Validation errors use text and an alert region.

### Navigation

The restrained white header contains Builder, Saved projects and Free & Pro links. Current navigation uses ink and a small green dot. The horizontal Template → Settings → Export / upload stepper uses numbered circles, fine desktop connectors and a dark filled current number. Step controls remain present during configuration. Validation gates entry to Export; changing steps moves keyboard focus to the workspace.

### Workflow choices

Three semantic radio labels present Text to video, Image to video and Reference to video, each with a geometric SVG, sans-serif title/description and written Free/Pro badge. Checked radios use selection mint and a forest border. Free enables text-to-video and visibly disables the two Pro workflows. The responsive list shifts from three upright choices to horizontal rows rather than shrinking the copy into narrow columns.

### Notebook plan

A rounded mint panel keeps the current workflow, preset, frame-adjusted duration, dimensions, runtime, image filename and LoRA state in view beside the workspace. Label/value rows align across the desktop panel, then become a two-column fact grid beneath the workspace on mobile. Save project and Open core notebook use transparent panel buttons. The plan explicitly locates GPU execution in the user's Colab account.

### Plan badges and comparison

Free and Pro badges are small muted pills. The Pro state says preview and includes a written no-billing notice. The Free & Pro page pairs white and mint rounded offerings: Free is $0; Pro says Preview available. It does not quote a paid subscription price or collect payment.

### Status, projects and handoff

A skip link, keyboard focus rings, alert region and polite fixed toast support accessible task feedback. Saved projects expose empty, open and two-step remove/keep states. Configuration storage is limited to 20 projects in this browser on this device; images and Google tokens are not stored, and files must be reselected after reopening when needed.

Notebook download and Google Drive upload remain separate paths. The export screen states that the notebook is editable and runs on the user's Colab account. The Drive preview asks for a Web application OAuth client ID and authorized origin, uses `drive.file`, keeps client ID/tokens in page memory and displays busy/success states. Live Google authorization and upload remain unverified; the presence of these UI states is not a claim of a live cloud test. Accounts, cloud sync, subscriptions and billing are not implemented.

## Do's and Don'ts

### Do:

- Do use the cool ground, forest actions and mint plan when extending browser screens.
- Do keep serif scale in editorial headings and task controls in Manrope.
- Do preserve the top stepper, visible radio state and keyboard focus treatment.
- Do follow the implemented breakpoints and let the plan flow below the workspace on mobile.
- Do show Free restrictions, Pro preview status and Colab execution in text.
- Do describe projects as device-local and require image reselection when the file is no longer in memory.

### Don't:

- Don't restore the rejected yellow browser rail or use the native legacy appendix as the browser palette.
- Don't repeat the large welcome heading above Settings or Export controls.
- Don't add decorative shadows to working surfaces or remove the reduced-motion path.
- Don't imply active billing, cloud sync, local inference or verified live Drive authorization/upload.
- Don't treat native logical-pixel measurements as browser CSS dimensions.
- Don't replace labelled SVG workflow choices with undocumented shipping raster artwork.


## Appendix: native Windows legacy reference

This preserved system applies only to the optional Windows companion. Its yellow rail, Georgia guidance, native widget behavior and logical-pixel measurements are historical platform documentation, not browser design tokens. No native UI changes are part of the browser redesign.

### Legacy token snapshot

```yaml
name: GoobleLab
description: Native Windows controls on a warm, chrome-yellow notebook builder.
colors:
  rail: "#f4c62a"
  accent: "#253d36"
  ground: "#f6f1e3"
  surface: "#fffdf7"
  summary: "#eee9dc"
  workflow-selected: "#fff9db"
  ink: "#17201f"
  muted: "#586052"
  border: "#ccc9b9"
  control-hover: "#efe6c6"
  control-active: "#e6dbb3"
  white: "#ffffff"
  error: "#a62626"
  warning: "#844f0c"
typography:
  title:
    fontFamily: "Segoe UI, Ubuntu-Light"
    fontSize: "28px"
  body:
    fontFamily: "Segoe UI, Ubuntu-Light"
    fontSize: "16px"
  button:
    fontFamily: "Segoe UI, Ubuntu-Light"
    fontSize: "16px"
  guidance:
    fontFamily: "Georgia"
    fontSize: "16px"
  template-title:
    fontFamily: "Segoe UI, Ubuntu-Light"
    fontSize: "18px"
  template-guidance:
    fontFamily: "Georgia"
    fontSize: "15px"
  small:
    fontFamily: "Segoe UI, Ubuntu-Light"
    fontSize: "13px"
rounded:
  square: "0px"
  control: "6px"
spacing:
  item: "10px"
  section: "16px"
  main-inset: "28px"
  rail-inset: "24px"
  summary-inset: "22px"
components:
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.white}"
    typography: "{typography.button}"
    rounded: "{rounded.control}"
    padding: "9px 14px"
  button-secondary:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    typography: "{typography.button}"
    rounded: "{rounded.control}"
    padding: "9px 14px"
  button-secondary-hover:
    backgroundColor: "{colors.control-hover}"
  button-secondary-active:
    backgroundColor: "{colors.control-active}"
  text-field:
    backgroundColor: "{colors.white}"
    textColor: "{colors.ink}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
  workflow:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    height: "105px"
  workflow-selected:
    backgroundColor: "{colors.workflow-selected}"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    height: "105px"
  progress-tab:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    typography: "{typography.button}"
    height: "44px"
  notebook-summary:
    backgroundColor: "{colors.summary}"
    textColor: "{colors.ink}"
    rounded: "{rounded.square}"
    padding: "22px"
    width: "235px"
```

### Legacy Overview

**Creative North Star: "Chrome manual"**

A bright yellow progress rail, warm reading surfaces and dark green actions give the optional notebook companion the character of a practical printed manual. Segoe UI carries the controls; Georgia carries short guidance. The notebook remains the core product, and GPU work happens in Colab.

The native source of truth is `desktop/src/main.rs`, with inherited widget behavior from the locked egui 0.37.0 dependency. Measurements use native logical pixels (egui points), scaled by desktop DPI; they are not CSS rem or fixed physical pixels.

**Key Characteristics:**

- Chrome-yellow orientation rail and warm neutral work surfaces.
- Native Windows typography with serif guidance.
- Flat panels, compact controls and a continuously visible notebook plan.
- Text, borders and selection states carry meaning without shipping raster artwork.

### Legacy Colors

The palette combines a strong yellow navigation field with muted paper surfaces and green-black text.

#### Primary

- **Chrome yellow** (`rail`): progress rail and unselected step backgrounds. Text and links on this field use ink.
- **Deep green** (`accent`): primary actions, hyperlinks, selected workflow borders, text selection and non-error status messages. Primary actions use white text.

#### Neutral

- **Warm ground** (`ground`): central work area and general panel ground.
- **Warm white** (`surface`): inactive buttons, workflow choices and current-step tab.
- **Summary paper** (`summary`): the notebook-plan pane.
- **Selected paper** (`workflow-selected`): selected workflow fill.
- **Ink** (`ink`): ordinary text, control labels and rail links.
- **Olive grey** (`muted`): supporting copy, guidance and summary labels.
- **Paper border** (`border`): inactive control and unselected workflow outlines.
- **Warm hover / pressed paper** (`control-hover`, `control-active`): inherited button states where a control does not supply its own explicit fill.
- **White** (`white`): primary-action text, selected text and the inherited egui text-field background.
- **Error red / warning brown** (`error`, `warning`): validation failures and experimental-preset guidance.

**The Rail Contrast Rule.** Rail text and rail hyperlinks use ink. The general green hyperlink color is used on the reading surfaces.

### Legacy Typography

**Control font:** Segoe UI, loaded from the Windows Fonts directory. The bundled egui proportional family, led by Ubuntu-Light, remains the fallback.

**Guidance font:** Georgia, also loaded from Windows. If Georgia cannot be loaded, the guidance family uses the proportional fallback family, including Segoe UI when available.

The sizes in the legacy token snapshot above are the actual native text sizes. Line height, glyph fallback and strong-text treatment remain egui behavior; there is no separate CSS weight, tracking or line-height scale.

- **Title:** screen headings.
- **Body / button:** editable values, control labels, buttons and ordinary explanatory copy.
- **Guidance:** the short introduction below a screen heading and the rail's product statement.
- **Template title / template guidance:** the two-level text inside workflow choices.
- **Small:** field help, summary labels, privacy notes and the rail footer.

**The Guidance Rule.** Use the serif family for short guidance; controls and editable text use the proportional UI family.

### Legacy Layout

The native window opens at 1100 × 760 and has a minimum inner size of 900 × 660. The progress rail is fixed at 205 wide, the notebook-plan pane at 235, and the central work pane absorbs the remaining width. Insets and the shared item spacing are recorded in the legacy token snapshot above. This is a desktop three-pane layout; no mobile layout or web breakpoints are implemented.

The work area and notebook plan scroll independently. The main work scrollbar is always visible, uses the inherited solid scrollbar style, and reserves space rather than floating over content. The primary navigation action sits below the main scroller. The work area reserves 82 of vertical footer space normally and 190 when a status message is present; long status text has its own scroller capped at 90.

Workflow choices span the available central width. Settings use full-width text fields and a two-column output grid with spacing of 22 horizontally and 12 vertically. Labels are placed above text fields; longer image and OAuth rows wrap.

### Legacy Elevation & Depth

The three main panels and workflow choices are flat and use tonal separation and outlines; they have no added shadows. Standard egui popups retain the dependency's light-theme shadow: offset 6 horizontally and 10 vertically, blur 8, spread 0, black alpha 25/255. This is a native popup default, not a CSS shadow contract. No custom motion system is implemented; standard widget animation inherits egui's 0.2-second setting.

### Legacy Shapes

Panel frames are square. The current-step surface extends to the rail's outer edge as a square warm-white backing, behind a normal rounded button. Buttons and editable controls retain egui's gently rounded control corners. Standard outlines are one logical pixel; selected workflow choices use a two-pixel deep-green outline. Workflow choices are full-width button surfaces, not image cards.

### Legacy Components

#### Buttons

Primary actions use deep green with explicit white text. Secondary controls use warm white, ink and the paper border. Both use the shared native button padding. Explicitly filled buttons retain their fill through interaction; generic secondary controls use the warm hover and pressed fills. Hover, focus and disabled behavior otherwise remain egui defaults. Busy work disables the work controls and progress navigation.

#### Inputs / Fields

Single-line text fields fill the available width and associate their visible label through egui's `labelled_by`. The prompt uses a seven-row multiline field. Fields retain the light theme's white background and native focus treatment; selected text uses deep green with white text. Presets and resolution use native egui combo boxes; duration and seed use drag-value controls. The optional LoRA controls sit in a collapsing section.

#### Navigation

Three numbered rail steps lead through Template, Settings and Export/upload. Each step has a 44-high target and no explicit border. The current step's warm-white backing continues to the rail edge. Validation gates entry to Export/upload and returns invalid configuration to Settings with a visible error. Back and next actions appear below the central scroller.

#### Workflow choices

Each choice combines a UI-font title with serif guidance in a 105-high button. The selected choice uses selected paper and the thicker accent outline; the rest use warm white and the standard paper border. Text-to-video, image-to-video and reference-image-to-video remain text-led choices.

#### Notebook plan

The fixed summary pane uses summary paper with a square frame. Small muted labels sit above stronger values, separated by generous vertical gaps. Workflow, preset, dimensions, duration, runtime, selected image and LoRA state remain visible while the central pane changes.

#### Status and handoff

Status copy appears below the main scroller: deep green for ordinary progress or success, red for errors. A spinner and explicit waiting or upload message accompany background work; sign-in exposes cancellation. Local export and Drive publishing are separate actions. The UI states that inference runs in Colab and that Google tokens remain in memory. Documentation of these states is not evidence of a completed live OAuth or browser upload test.

### Legacy Do's and Don'ts

#### Do:

- **Do** retain ink text and links on the yellow rail.
- **Do** keep control text in the UI family and short guidance in the guidance family.
- **Do** preserve the independent work and summary scrollers at the minimum window size.
- **Do** keep the notebook plan and the Colab handoff explicit.
- **Do** use native control behavior and the implemented palette when extending this companion.

#### Don't:

- **Don't** treat the working Chrome manual direction as a locked user selection.
- **Don't** describe native logical pixels as web rem or add unimplemented mobile breakpoints.
- **Don't** replace observable native states with invented CSS effects, gradients or card shadows.
- **Don't** imply that the desktop app runs local inference or that live Google sign-in has been verified.
