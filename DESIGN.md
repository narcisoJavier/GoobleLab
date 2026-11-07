---
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
  web-primary-hover: "#344e43"
  web-step-hover: "#f9da70"
  web-disabled-button: "#eeeade"
  web-disabled-text: "#76786c"
  web-disabled-border: "#d9d5c6"
  web-disabled-field: "#f0ecdf"
  web-disabled-field-text: "#727568"
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
  web-body:
    fontFamily: "Segoe UI, system-ui, sans-serif"
    fontSize: "16px"
    lineHeight: 1.5
  web-title:
    fontFamily: "Segoe UI, system-ui, sans-serif"
    fontSize: "clamp(27px, 2.3vw, 35px)"
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: "-0.025em"
  web-guidance:
    fontFamily: "Georgia, serif"
    fontSize: "17px"
    lineHeight: 1.6
  web-button:
    fontFamily: "Segoe UI, system-ui, sans-serif"
    fontSize: "16px"
    fontWeight: 550
    lineHeight: 1.4
  web-field-label:
    fontFamily: "Segoe UI, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 600
  web-badge:
    fontFamily: "Segoe UI, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 650
rounded:
  square: "0px"
  control: "6px"
  web-badge: "4px"
spacing:
  item: "10px"
  section: "16px"
  main-inset: "28px"
  rail-inset: "24px"
  summary-inset: "22px"
  web-header-gutter: "32px"
  web-workspace-top: "38px"
  web-workflow-inset: "22px"
  web-item-gap: "12px"
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
  web-button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.white}"
    typography: "{typography.web-button}"
    rounded: "{rounded.control}"
    padding: "9px 16px"
  web-button-primary-hover:
    backgroundColor: "{colors.web-primary-hover}"
  web-button-secondary:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    typography: "{typography.web-button}"
    rounded: "{rounded.control}"
    padding: "9px 16px"
  web-button-secondary-hover:
    backgroundColor: "{colors.control-hover}"
  web-field:
    backgroundColor: "{colors.white}"
    textColor: "{colors.ink}"
    typography: "{typography.web-body}"
    rounded: "{rounded.control}"
    padding: "10px 12px"
  web-workflow:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    padding: "22px"
  web-workflow-selected:
    backgroundColor: "{colors.workflow-selected}"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    padding: "21px"
  web-badge-free:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    typography: "{typography.web-badge}"
    rounded: "{rounded.web-badge}"
    padding: "3px 9px"
  web-badge-pro:
    backgroundColor: "{colors.rail}"
    textColor: "{colors.ink}"
    typography: "{typography.web-badge}"
    rounded: "{rounded.web-badge}"
    padding: "3px 9px"
---

# Design System: GoobleLab

## Overview

The original descriptions below are the preserved **native legacy reference**. Each **Browser extension** subsection documents the current primary browser interface and overrides platform-specific behavior only for that interface.

**Creative North Star: "Chrome manual"**

A bright yellow progress rail, warm reading surfaces and dark green actions give the optional notebook companion the character of a practical printed manual. Segoe UI carries the controls; Georgia carries short guidance. The notebook remains the core product, and GPU work happens in Colab.

This documents the implemented working direction after the user's request for a bolder design. The user has not locked an exploration card; this is an observed system, not a claim of final visual approval. The source of truth is `desktop/src/main.rs`, with inherited widget behavior from the locked egui 0.37.0 dependency. Measurements use native logical pixels (egui points), scaled by desktop DPI; they are not CSS rem or fixed physical pixels.

**Key Characteristics:**

- Chrome-yellow orientation rail and warm neutral work surfaces.
- Native Windows typography with serif guidance.
- Flat panels, compact controls and a continuously visible notebook plan.
- Text, borders and selection states carry meaning without shipping raster artwork.

### Browser extension

The browser is now the primary interface; the Windows companion documented above is preserved as a native legacy reference. The browser extends the same Chrome manual working direction, without a new visual world or claim of a locked user selection. `web/style.css` owns browser styling, `web/index.html` owns the shell, and `web/app.mjs` supplies the visible builder, projects and plan states. All browser measurements below are CSS pixels; the native logical-pixel measurements above remain unchanged.

The browser retains Template → Settings → Export/upload, with a header for Builder, Saved projects and Free & Pro. Free configures a basic notebook; Pro is an advanced customization evaluation preview. Both produce editable notebooks run in the user's own Colab account. No shipping raster assets are used: the brand icon and favicon are SVG, and selected image previews are user-provided media.

## Colors

The palette combines a strong yellow navigation field with muted paper surfaces and green-black text.

### Primary

- **Chrome yellow** (`rail`): progress rail and unselected step backgrounds. Text and links on this field use ink.
- **Deep green** (`accent`): primary actions, hyperlinks, selected workflow borders, text selection and non-error status messages. Primary actions use white text.

### Neutral

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

### Browser extension

Browser custom properties reuse the existing rail, accent, ground, surface, summary, selected-paper, ink, muted, border, error and warm-hover colors. The namespaced web primitives add only colors actually present in the CSS: the darker primary hover, yellow step hover, and disabled-button and disabled-field treatments. Free badges use warm white; Pro badges use chrome yellow. Browser rail headings and step labels use ink; the rail's inline link inherits accent from the browser link rule. The ink-only rail-link rule above describes the native legacy implementation.

## Typography

**Control font:** Segoe UI, loaded from the Windows Fonts directory. The bundled egui proportional family, led by Ubuntu-Light, remains the fallback.

**Guidance font:** Georgia, also loaded from Windows. If Georgia cannot be loaded, the guidance family uses the proportional fallback family, including Segoe UI when available.

The sizes in the frontmatter are the actual native text sizes. Line height, glyph fallback and strong-text treatment remain egui behavior; there is no separate CSS weight, tracking or line-height scale.

- **Title:** screen headings.
- **Body / button:** editable values, control labels, buttons and ordinary explanatory copy.
- **Guidance:** the short introduction below a screen heading and the rail's product statement.
- **Template title / template guidance:** the two-level text inside workflow choices.
- **Small:** field help, summary labels, privacy notes and the rail footer.

**The Guidance Rule.** Use the serif family for short guidance; controls and editable text use the proportional UI family.

### Browser extension

The browser uses the installed Segoe UI / system UI / sans-serif stack, rather than loading Windows font files. Body text is 16px with a 1.5 line height; Georgia introductions are 17px with a 1.6 line height. Workspace titles scale from 27px to 35px with `clamp`, weight 600 and tight tracking; full-page titles use 34px. The namespaced browser frontmatter records the actual title, body, guidance, button, label and badge roles. Workflow titles are 19px, reduced to 18px at the compact desktop breakpoint; their serif descriptions are 15px. Browser strong labels and weights are CSS declarations, separate from the native text treatment.

## Layout

The native window opens at 1100 × 760 and has a minimum inner size of 900 × 660. The progress rail is fixed at 205 wide, the notebook-plan pane at 235, and the central work pane absorbs the remaining width. Insets and the shared item spacing are recorded in the frontmatter. This is a desktop three-pane layout; no mobile layout or web breakpoints are implemented.

The work area and notebook plan scroll independently. The main work scrollbar is always visible, uses the inherited solid scrollbar style, and reserves space rather than floating over content. The primary navigation action sits below the main scroller. The work area reserves 82 of vertical footer space normally and 190 when a status message is present; long status text has its own scroller capped at 90.

Workflow choices span the available central width. Settings use full-width text fields and a two-column output grid with spacing of 22 horizontally and 12 vertically. Labels are placed above text fields; longer image and OAuth rows wrap.

### Browser extension

The default desktop header is at least 80px high, with 16px vertical and 32px horizontal padding. Under it, the builder uses a 220px rail, a flexible workspace and a 282px notebook plan. The workspace has 38px top padding and a horizontal inset of `clamp(24px, 3.8vw, 64px)`; the plan uses 33px vertical and 25px horizontal padding. The footer is at least 57px high, and the shell's minimum height is `calc(100vh - 137px)`. The browser uses normal document flow and scrolling; native independent-pane scrolling is not a browser behavior.

- At **1450px and wider**, rail and plan widths become 240px and 310px; the workspace is capped at 1100px and centered within its column.
- At **1050px and narrower**, the rail and plan become 185px and 235px, header padding becomes 15px 22px, and the workspace uses 30px 24px padding. Workflow insets reduce from 22px to 18px, or 17px when selected.
- At **850px and narrower**, the header navigation wraps to a full-width second row. The builder becomes one column: horizontal step buttons, workspace, then notebook plan. The rail introduction and footer note are hidden. Plan values form two columns; plan actions form two columns. Free/Pro comparison offerings stack vertically.
- At **450px and narrower**, workspace insets reduce to 18px, paired fields and plan actions stack, workflow badges move below the description, step text becomes 12px, and page headings/actions and image-selection content stack. The header uses 15px 18px padding.

Saved projects and Free & Pro use a centered page container capped at 1120px. Project tables retain their columns inside a horizontally scrollable table wrapper. The plan comparison uses two adjacent flat offerings on desktop and a vertical sequence at the mobile breakpoint. Screenshot dimensions are review artifacts; CSS viewport width determines which breakpoint applies.

## Elevation & Depth

The three main panels and workflow choices are flat and use tonal separation and outlines; they have no added shadows. Standard egui popups retain the dependency's light-theme shadow: offset 6 horizontally and 10 vertically, blur 8, spread 0, black alpha 25/255. This is a native popup default, not a CSS shadow contract. No custom motion system is implemented; standard widget animation inherits egui's 0.2-second setting.

### Browser extension

Browser panels, workflow choices and plan offerings remain flat with one-pixel borders and tonal separation. The fixed status toast is the implemented raised exception: `0 8px 28px #17201f26`. It uses an accent fill, white text, a 6px radius and an error-red fill for failures. With reduced motion not requested, buttons transition background and border colors over 120ms with `ease`, workflows transition border color over 120ms, and toast entry translates upward by 8px over 160ms with `ease-out`. These effects are inside `prefers-reduced-motion: no-preference`; no decorative motion is added outside that condition.

## Shapes

Panel frames are square. The current-step surface extends to the rail's outer edge as a square warm-white backing, behind a normal rounded button. Buttons and editable controls retain egui's gently rounded control corners. Standard outlines are one logical pixel; selected workflow choices use a two-pixel deep-green outline. Workflow choices are full-width button surfaces, not image cards.

### Browser extension

Browser controls, workflow choices, notices and image-selection containers reuse the 6px corner radius. Badges and selected image thumbnails use 4px corners. Desktop step buttons round only the left corners and meet the rail edge on the right; at the one-column breakpoint, all step corners are rounded. Selected workflow borders increase from one to two pixels while their padding drops by one pixel, preserving the occupied size. Focus-visible elements use an accent outline of 3px with a 4px offset.

## Components

### Buttons

Primary actions use deep green with explicit white text. Secondary controls use warm white, ink and the paper border. Both use the shared native button padding. Explicitly filled buttons retain their fill through interaction; generic secondary controls use the warm hover and pressed fills. Hover, focus and disabled behavior otherwise remain egui defaults. Busy work disables the work controls and progress navigation.

### Inputs / Fields

Single-line text fields fill the available width and associate their visible label through egui's `labelled_by`. The prompt uses a seven-row multiline field. Fields retain the light theme's white background and native focus treatment; selected text uses deep green with white text. Presets and resolution use native egui combo boxes; duration and seed use drag-value controls. The optional LoRA controls sit in a collapsing section.

### Navigation

Three numbered rail steps lead through Template, Settings and Export/upload. Each step has a 44-high target and no explicit border. The current step's warm-white backing continues to the rail edge. Validation gates entry to Export/upload and returns invalid configuration to Settings with a visible error. Back and next actions appear below the central scroller.

### Workflow choices

Each choice combines a UI-font title with serif guidance in a 105-high button. The selected choice uses selected paper and the thicker accent outline; the rest use warm white and the standard paper border. Text-to-video, image-to-video and reference-image-to-video remain text-led choices.

### Notebook plan

The fixed summary pane uses summary paper with a square frame. Small muted labels sit above stronger values, separated by generous vertical gaps. Workflow, preset, dimensions, duration, runtime, selected image and LoRA state remain visible while the central pane changes.

### Status and handoff

Status copy appears below the main scroller: deep green for ordinary progress or success, red for errors. A spinner and explicit waiting or upload message accompany background work; sign-in exposes cancellation. Local export and Drive publishing are separate actions. The UI states that inference runs in Colab and that Google tokens remain in memory. Documentation of these states is not evidence of a completed live OAuth or browser upload test.

### Browser extension

**Actions and fields.** Browser buttons have a minimum height of 43px, 9px 16px padding and the namespaced browser button role. Primary hover uses the actual darker green CSS color; secondary hover uses warm paper. Small actions are at least 35px high. Inputs, selects and textareas are full width with 10px 12px padding and at least 44px height; the prompt textarea starts at 180px and resizes vertically. Visible labels associate with controls. A skip link, keyboard-visible focus outline, alert region and polite status toast are implemented.

**Workflow and plan states.** Workflow choices are semantic radio labels with a visible radio, title, serif description and Free/Pro badge. Free selects text-to-video and disables image and reference options. The selected choice uses selected paper and the thicker accent border. Free keeps advanced generation controls visibly disabled, with an explicit Preview controls action. Pro preview enables those controls and displays both a yellow plan badge and a written no-billing notice. The comparison page shows Free at $0 and Pro as Preview available; it does not quote a subscription price or collect payment. Switching back to Free preserves advanced settings for a later Pro preview while the active Free notebook uses the basic defaults.

**Notebook plan and projects.** The browser plan reflects the active workflow, preset, frame-adjusted duration, dimensions, runtime, image filename and LoRA status. Save project stores up to 20 configurations in this browser on this device. The Saved projects page provides empty, open and two-step remove/keep states. Images and Google tokens are not persisted; an image filename can remain in the saved configuration, and reopening a project asks the user to reselect the file for Drive upload. Cloud accounts and synchronization are not enabled.

**Export and Drive preview.** Notebook download is available without Google login. Image workflows ask for the image in Colab when downloaded. Drive upload is a separate preview with a user-provided Web application OAuth client ID and authorized JavaScript origin; Desktop OAuth JSON is explicitly identified as belonging to the legacy Windows app. The client ID and tokens remain in page memory, and authorization uses `drive.file`. Drive operations disable relevant controls and show a busy message. The implemented success state supplies Colab and Drive links. Live Google authorization/upload remains unverified; these documented states do not claim cloud functionality was exercised. Inference remains in Colab.

## Do's and Don'ts

### Do:

- **Do** retain ink text and links on the yellow rail.
- **Do** keep control text in the UI family and short guidance in the guidance family.
- **Do** preserve the independent work and summary scrollers at the minimum window size.
- **Do** keep the notebook plan and the Colab handoff explicit.
- **Do** use native control behavior and the implemented palette when extending this companion.

### Don't:

- **Don't** treat the working Chrome manual direction as a locked user selection.
- **Don't** describe native logical pixels as web rem or add unimplemented mobile breakpoints.
- **Don't** replace observable native states with invented CSS effects, gradients or card shadows.
- **Don't** imply that the desktop app runs local inference or that live Google sign-in has been verified.

### Browser extension

- **Do** preserve the shared warm yellow palette and the existing guided sequence when adding browser screens.
- **Do** use the actual CSS breakpoints and browser focus treatment; retain the separate native legacy measurements above.
- **Do** show Free restrictions and Pro preview status in text as well as control states and badges.
- **Do** describe projects as device-local and require image reselection when the file is no longer in memory.
- **Don't** claim an approved new visual identity, active billing, cloud sync or a verified live Drive upload.
- **Don't** apply the native three-pane scrolling model to the responsive browser document.
- **Don't** replace SVG branding or text-led workflow choices with undocumented shipping raster artwork.
