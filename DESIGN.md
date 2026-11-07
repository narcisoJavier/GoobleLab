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
---

# Design System: GoobleLab

## Overview

**Creative North Star: "Chrome manual"**

A bright yellow progress rail, warm reading surfaces and dark green actions give the optional notebook companion the character of a practical printed manual. Segoe UI carries the controls; Georgia carries short guidance. The notebook remains the core product, and GPU work happens in Colab.

This documents the implemented working direction after the user's request for a bolder design. The user has not locked an exploration card; this is an observed system, not a claim of final visual approval. The source of truth is `desktop/src/main.rs`, with inherited widget behavior from the locked egui 0.37.0 dependency. Measurements use native logical pixels (egui points), scaled by desktop DPI; they are not CSS rem or fixed physical pixels.

**Key Characteristics:**

- Chrome-yellow orientation rail and warm neutral work surfaces.
- Native Windows typography with serif guidance.
- Flat panels, compact controls and a continuously visible notebook plan.
- Text, borders and selection states carry meaning without shipping raster artwork.

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

## Layout

The native window opens at 1100 × 760 and has a minimum inner size of 900 × 660. The progress rail is fixed at 205 wide, the notebook-plan pane at 235, and the central work pane absorbs the remaining width. Insets and the shared item spacing are recorded in the frontmatter. This is a desktop three-pane layout; no mobile layout or web breakpoints are implemented.

The work area and notebook plan scroll independently. The main work scrollbar is always visible, uses the inherited solid scrollbar style, and reserves space rather than floating over content. The primary navigation action sits below the main scroller. The work area reserves 82 of vertical footer space normally and 190 when a status message is present; long status text has its own scroller capped at 90.

Workflow choices span the available central width. Settings use full-width text fields and a two-column output grid with spacing of 22 horizontally and 12 vertically. Labels are placed above text fields; longer image and OAuth rows wrap.

## Elevation & Depth

The three main panels and workflow choices are flat and use tonal separation and outlines; they have no added shadows. Standard egui popups retain the dependency's light-theme shadow: offset 6 horizontally and 10 vertically, blur 8, spread 0, black alpha 25/255. This is a native popup default, not a CSS shadow contract. No custom motion system is implemented; standard widget animation inherits egui's 0.2-second setting.

## Shapes

Panel frames are square. The current-step surface extends to the rail's outer edge as a square warm-white backing, behind a normal rounded button. Buttons and editable controls retain egui's gently rounded control corners. Standard outlines are one logical pixel; selected workflow choices use a two-pixel deep-green outline. Workflow choices are full-width button surfaces, not image cards.

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
