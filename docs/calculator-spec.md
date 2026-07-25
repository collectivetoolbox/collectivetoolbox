# Legacy Calculator Comprehensive UI & Asset Specification

This document provides a comprehensive specification of the UI layout, tab hierarchy, visual styles, control positioning, and asset inventory representing the **superset** of features across all versions in `old/calculator` (including Calculator 4.0, Assistance & Errors forms, the `6r2` Unique Random Number Generator sidecar, and the `R. P. S.` Rock-Paper-Scissors v1.2 sidecar).

---

## 1. Architectural Scope & Application Hierarchy

The complete Calculator application suite consists of three interconnected component executables/forms:
1. **Calculator 4.0 Main Application** (`Form1`): Multi-tab primary calculation suite.
2. **Calculator Assistance & Errors System** (`Form2` & `Form3`): Modal/non-modal help and error explanation screens.
3. **Rock, Paper, and Scissor (R. P. S.) Sidecar Game (v1.2)**: Bundled game with score tracking and bitmap action triggers.
4. **Unique Random Number Generator (6r2 v1.0)**: Sidecar utility generating non-repeating triplets of random integers.

---

## 2. Calculator 4.0 Main UI Specification (`Form1`)

### Top-Level Window Configuration
- **Title**: `Calculator`
- **Menu Bar (`mnuProgram` / `MainMenu1`)**:
  - `mnuCalc` ("Calculator"): Dropdown menu.
  - `mnuHelp` ("Assistance"): Triggers help/documentation dialog.
  - `mnuAbout` ("About..."): Displays copyright and version popups.
  - `mnuExit` ("Exit"): Application termination.
- **Main Container**: `tctCalc` (`TabControl`, size ~750x550px) hosting 10 primary feature tabs.

---

### Tab Breakdown

#### Tab 1: `tabMake` — Title: `"Make"`
- **Purpose**: Primary interactive evaluator supporting basic arithmetic, modulo, integer division, and powers.
- **Controls & Layout**:
  - `txtN1` (`TextBox`): Input for Operand 1 (left number).
  - `txtN2` (`TextBox`): Input for Operand 2 (right number).
  - `txtFn` (`TextBox`): Manual operator symbol string (`+`, `-`, `*`, `/`, `\`, `Mod`, `^`).
  - `grpChooseOne` (`GroupBox`, Text: `"Choose One:"`):
    - `radN` (`RadioButton`, Text: `"+"`): Selects addition.
    - `radS` (`RadioButton`, Text: `"-"`): Selects subtraction.
    - `radX` (`RadioButton`, Text: `"x"` / `"*"`): Selects multiplication.
    - `radD` (`RadioButton`, Text: `"/"`): Selects floating-point division.
    - `radBk` (`RadioButton`, Text: `"\"`): Selects integer division.
    - `radMod` (`RadioButton`, Text: `"Mod"`): Selects modulo / remainder division.
    - `radE` (`RadioButton`, Text: `"^"`): Selects exponentiation.
    - `radAC` (`RadioButton`, Text: `"AC"`): All Clear operator reset.
  - `btnEuqlas` / `btnEuqals` (`Button`, Text: `"Evaluate"`): Executes computation and updates result string.
  - `lblYourAnswer` (`Label`, Text: `"{ }"`): High-contrast result display label.
  - `btnClearAll` (`Button`, Text: `"Clear All"`): Resets inputs (requires unlocked session).
  - `btnExti` (`Button`, Text: `"Quit"`): Exits application (requires unlocked session).
  - `CheckBox1` (`CheckBox`, Text: `"Locked?"`): Toggles session lock.
  - `txtSP1` (`TextBox`, Text: `"Scratch Pad"`): Multiline scratch notes area.

#### Tab 2: `tabPrime` — Title: `"Prime verification"`
- **Purpose**: Verifies whether an integer is prime and calculates factors.
- **Controls & Layout**:
  - `txtNum1` (`TextBox`): Input field for candidate integer.
  - `btn1` (`Button`, Text: `"Test"`): Executes primality test loop.
  - `lbl1` (`Label`): Output message (`"This number is prime."` or `"This number is not prime. Two factors of this number are X and Y."`).
  - `lblFactors` (`Label`): Displays factor summary string.
  - `Button1` (`Button`, Text: `"About this software..."`): Sub-dialog trigger.

#### Tab 3: `tabRand` — Title: `"Random Numbers"`
- **Purpose**: Generates random numbers across 8 magnitude scale factors.
- **Controls & Layout**:
  - `grpRan` (`GroupBox`, Text: `"Random Numbers:"`):
    - `lblRan1`: Outputs $Rnd()$ ($[0.0, 1.0)$).
    - `lblRan2`: Outputs $Rnd() \times 5$.
    - `lblRan3`: Outputs $Rnd() \times 10$.
    - `lblRan4`: Outputs $Rnd() \times 50$.
    - `lblRan5`: Outputs $Rnd() \times 100$.
    - `lblRan6`: Outputs $Rnd() \times 500$.
    - `lblRan7`: Outputs $Rnd() \times 1000$.
    - `lblRan8`: Outputs $Rnd() \times 5000$.
  - `btnRand` (`Button`, Text: `"Refresh random numbers"`): Re-runs random number generator sequence.

#### Tab 4: `tabSqRt` — Title: `"Square Root"`
- **Purpose**: Computes real and complex square roots.
- **Controls & Layout**:
  - `txtSqRt` (`TextBox`): Numeric input value.
  - `btnSqRt` (`Button`, Text: `"Find Square Root"`): Calculates $\sqrt{x}$. For $x < 0$, formats as $\sqrt{-x}\,i$.
  - `lblYourAnswer` (`Label`): Display output.

#### Tab 5: `TabPage1` — Title: `"Temperature"`
- **Purpose**: Converts temperature between Fahrenheit and Celsius.
- **Controls & Layout**:
  - `txtFahrenhe` (`TextBox`): Input temperature value.
  - `btnFahrenhe` (`Button`, Text: `"Change to Fahrenheit"`): Converts Celsius input to Fahrenheit.
  - `btnCelsius` (`Button`, Text: `"Change to Celsius"`): Converts Fahrenheit input to Celsius.
  - `lblThetempe` (`Label`, Text: `"The temperature in"`).
  - `lblTempName` (`Label`, Text: `"Fahrenheit"` / `"Celsius"`).
  - `lblIs` (`Label`, Text: `"is"`).
  - `lblAns` (`Label`): Converted numeric result.
  - `lblO` (`Label`, Text: `"O"`): Degree symbol representation.
  - `lblKind` (`Label`, Text: `"F"` / `"C"`).
  - `lblEnterate` (`Label`, Text: `"Enter a temperature."`).

#### Tab 6: `tabPeri` — Title: `"Perimeter"`
- **Purpose**: Computes perimeter of a rectangle.
- **Controls & Layout**:
  - `txtRectPeri1` (`TextBox`): Rectangle base length.
  - `txtRectPeri2` (`TextBox`): Rectangle height.
  - `lblFindperi` (`Label`, Text: `"Find the perimeter of a rectangle with base"`).
  - `lblAndheighth` (`Label`, Text: `"and heighth"`).
  - `btnGetperi` (`Button`, Text: `"Get perimeter"`): Calculates $2 \times (base + height)$.

#### Tab 7: `tabAbout` — Title: `"About"`
- **Purpose**: Displays application metadata, copyright, and author contact.
- **Controls & Layout**:
  - `lblCalc` (`Label`, Text: `"Calculator"`).
  - `lblVersion` (`Label`, Text: `"Version 5.0"`).
  - `Label9` (`Label`, Text: `"Copyright Collective Toolbox Authors."`).
  - `lnkWeb` (`LinkLabel`, Text: `"https://collectivetoolbox.com/ ~ info@collectivetoolbox.com"`).

#### Tab 8: `tabUse` — Title: `"Constants"`
- **Purpose**: Provides quick constant insertion helpers.
- **Controls & Layout**:
  - `btnPi` (`Button`, Text: `"Pi"`): Inserts $\pi \approx 3.141592654$.
  - `btneconst` (`Button`, Text: `"e"`): Inserts Euler's constant $e \approx 2.718281828$.

#### Tab 9: `tabHelp` — Title: `"Assistance"`
- **Purpose**: Nested troubleshooting tab view.
- **Container**: `TabControl1` hosting:
  - `TabPage2` ("Errors"):
    - `Panel1`: Container with `PictureBox1` (error graphic diagram), `PictureBox3`, and `lblErr1WhatToDo` (`Label`, Text: `"What To Do: Click 'Quit', then restart the program and enter a smaller number."`).
  - `TabPage3` ("Incorrect Answers"):
    - Contains labels `Label1` through `Label13` explaining window sizing, text box digit truncation, and overflow conditions.

#### Tab 10: `tabAreas` — Title: `" Area"`
- **Purpose**: Geometry area calculator for circles and rectangles.
- **Controls & Layout**:
  - **Circle Area Sub-Section**:
    - `lblAreabyRad` (`Label`, Text: `"Find the area of a circle with radius"`).
    - `txtRadiusValue` (`TextBox`): Radius input.
    - `btnArea` (`Button`, Text: `"Get area"`): Computes $\pi \times r^2$.
  - **Rectangle Area Sub-Section**:
    - `lblRect` (`Label`, Text: `"Find the area of a rectangle with base"`).
    - `txtBase` (`TextBox`): Base length.
    - `lblHeigth` (`Label`, Text: `"and heighth"`).
    - `txtHeighth` (`TextBox`): Height length.
    - `btnRectArea` (`Button`, Text: `"Get area"`): Computes $base \times height$.
  - **Unit Radio Buttons**: `radQ1` (`a-ft^2`), `radQ9` (`A-ft^2`), `RadioButton3` (`yd^2-ft^2`), `RadioButton4` (`m^2-ft^2`), `RadioButton6` (`rd^2`).

---

## 3. Sidecar Applications Specification

### A. Rock, Paper, and Scissor (R. P. S. v1.2)
- **Executable Project**: `Calculator_ALL/R. P. S`
- **Main Window Title**: `Rock, Paper, and Scissor`
- **Menu (`mnuRPS`)**:
  - `mnuNew` ("New session..."): Resets score counters (`intW`, `intL`, `intD`).
  - `mnuExit` ("Exit"): Terminates application.
  - `MenuItem1` ("About RPS"): Displays info dialog `"RPS version 1.2. This is a subsidiary application of Calculator 2.0."`.
- **Tab Control (`tctGame`)**:
  - `tabGame` ("Game"):
    - Radio Buttons: `radRock` ("rock"), `radPaper` ("paper"), `radScissor` ("scissor").
    - `btnGo` (`Button`, Text: `"Go"`): Generates computer choice ($1 \dots 3$) and evaluates winner.
    - Status Labels: `lblYourDecision` ("You played: ..."), `lblCompDecision` ("Computer played: ..."), `lblWon` ("You Win" / "You Lose" / "Draw").
    - Score Counters: `lblWins` ("0 Wins"), `lblDraws` ("0 Draws"), `lblLosses` ("0 Losses").
    - Image Buttons: `pctNew` (uses `qb.bmp` for new session), `pctQuit` (uses `untitled.bmp` for quit).
  - `tabAbout` ("About"):
    - `lblRPS` ("Rock, Paper, and Scissor"), `lblVersion` ("Version 4.0"), `lblCpr` ("Copyright 2008").

### B. Unique Random Number Generator (6r2 v1.0)
- **Executable Project**: `6r2`
- **Main Window Title**: `Generate unique random numbers`
- **Menu (`mmn1`)**:
  - `mnuFile` -> `mnuNew` ("New session..."), `mnuExit` ("Exit").
- **Layout & Controls**:
  - `lbl1`, `lbl2`, `lbl3` (`Label`): Displays three non-identical integers in range $[0, 5]$.
  - `lblNumber` (`Label`): Displays iteration count.
  - `btnGetNew` (`Button`, Text: `"Generate unique random numbers"`): Loops generation until `int1 != int2`, `int2 != int3`, and `int3 != int1`.
  - `chkLock` (`CheckBox`, Text: `"Lock session"`): Prevents resetting unless unchecked.
  - `Button1` (`Button`, Text: `"About this software..."`): Info popup `"Unique random number generator; version 1.0. This is a subsidiary application of Calculator 2.0."`.

---

## 4. Complete Asset Inventory

### Image Files & Bitmaps

| Filename | Dimensions / Size | Primary Usage & Placement |
| :--- | :--- | :--- |
| `c10.bmp` | 65,590 bytes | Background graphic framing asset for Calculator forms. |
| `c11.bmp` | 65,590 bytes | Background graphic framing asset for main tabs. |
| `c12.bmp` | 65,590 bytes | Background graphic framing asset for sub-panels. |
| `c1a0.bmp` | 65,590 bytes | Additional panel framing graphic asset. |
| `Picture 1.png` | 12,505 bytes | Application branding image / screenshot. |
| `qb.bmp` | 3,126 bytes | Bitmap button graphic for RPS New Game trigger (`pctNew`). |
| `untitled.bmp` | 3,126 bytes | Bitmap button graphic for RPS Quit trigger (`pctQuit`). |
| `Icon1.ico` | 766 bytes | Primary application window icon. |
| `Icon2.ico` | 766 bytes | Secondary window titlebar icon. |
| `calcicon14.png` | 509 bytes | 14x14 icon asset. |
| `calcicon16.png` | 353 bytes | 16x16 icon asset. |
| `calcicon32.png` | 342 bytes | 32x32 icon asset. |
| `calcicon64.png` | 3,017 bytes | 64x64 icon asset. |
| `calcicon192.png` | 15,628 bytes | 192x192 icon asset. |
| `calcicon360.png` | 33,216 bytes | 360x360 icon asset. |
| `86.png` | 35,152 bytes | Graphic asset. |

### RESX Resources
- **`Form1.resx`**: Contains background image binary blobs for `tabMake.BackgroundImage`, `tabPrime.BackgroundImage`, `tabRand.BackgroundImage`, `tabSqRt.BackgroundImage`, `TabPage1.BackgroundImage`, `tabPeri.BackgroundImage`, `tabAbout.BackgroundImage`, `tabUse.BackgroundImage`, `tabHelp.BackgroundImage`, `TabPage2.BackgroundImage`, `Panel1.BackgroundImage`, `TabPage3.BackgroundImage`, `tabAreas.BackgroundImage`.
- **`Form3.resx`**: Contains binary blobs for `lblErr.Image` (Error header graphic) and `PictureBox1.Image` / `PictureBox1.BackgroundImage` (Error diagram).

---

## 5. Visual Styles & Theme Specification

- **Typography**:
  - Default Form Font: Microsoft Sans Serif 8.25pt Regular.
  - Heading & Special Labels: `Eras Light ITC` 18pt Regular (`lblErr`), `Eras Light ITC` 9.75pt Regular (`lblErr1WhatToDo`).
  - Button Special Styles: `Georgia` 12pt Italic (`btnArea`, ForeColor: `DarkSlateBlue`).
- **Color Palette**:
  - Primary Label ForeColor: `DarkSlateBlue`.
  - Panel & Tab Backgrounds: `Transparent` over embedded bitmaps or standard WinForms control color (`Control`).
  - Result Display: High contrast text enclosed in brackets (`"{ }"`).
- **Control Borders**: `BorderStyle.Fixed3D` for panel frames and status readouts.
