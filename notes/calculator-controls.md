# Legacy Calculator UI Controls Catalog

This document details the complete set of WinForms UI control types utilized across all versions of the legacy calculator application suite in `old/calculator` (including Calculator 4.0, Assistance & Errors forms, the `6r2` Unique Random Number Generator, and the `R. P. S.` Rock-Paper-Scissors sidecar game).

---

## Control Types Summary

| Control Class | WinForms Type | Primary Purpose & Usage in Applications |
| :--- | :--- | :--- |
| **TabControl** | `System.Windows.Forms.TabControl` | Multi-page container providing tabbed navigation across calculator feature domains and help subsystems. |
| **TabPage** | `System.Windows.Forms.TabPage` | Individual content pages contained within `TabControl` instances. |
| **Button** | `System.Windows.Forms.Button` | Interactive action triggers for evaluations, reset/clearing, constant insertions, unit conversions, and navigation. |
| **TextBox** | `System.Windows.Forms.TextBox` | Input fields for numeric operands, operators, scratch pads, and formula parameters. |
| **Label** | `System.Windows.Forms.Label` | Text headers, static labels, error instructions, result displays, dynamic value readouts, and game counters. |
| **LinkLabel** | `System.Windows.Forms.LinkLabel` | Clickable hyperlink displaying web URL and contact info in the About tab. |
| **RadioButton** | `System.Windows.Forms.RadioButton` | Exclusive selection controls for arithmetic operations, measurement unit selections, and game choices. |
| **CheckBox** | `System.Windows.Forms.CheckBox` | Toggle control used to lock/unlock sessions to prevent accidental resets or application exits. |
| **GroupBox** | `System.Windows.Forms.GroupBox` | Bordered frame containers grouping related radio buttons, random number readouts, or options. |
| **Panel** | `System.Windows.Forms.Panel` | Layout containers organizing sub-elements and background styling within help and operator views. |
| **PictureBox** | `System.Windows.Forms.PictureBox` | Visual containers displaying embedded background graphics, branding images, error icons, and bitmap action buttons. |
| **MainMenu** | `System.Windows.Forms.MainMenu` | Application window top-level menu bar container. |
| **MenuItem** | `System.Windows.Forms.MenuItem` | Menu options under file, application, assistance, and game menu structures. |

---

## Detailed Control Descriptions & Instances

### 1. TabControl & TabPage
- **`tctCalc`**: Main tab control in Calculator 4.0 hosting 10 primary feature tabs:
  - `tabMake` ("Make"): Primary arithmetic calculator interface.
  - `tabPrime` ("Prime verification"): Prime number tester and factor finder.
  - `tabRand` ("Random Numbers"): Multi-range random number generator readout.
  - `tabSqRt` ("Square Root"): Real and complex square root evaluator.
  - `TabPage1` ("Temperature"): Celsius $\leftrightarrow$ Fahrenheit temperature converter.
  - `tabPeri` ("Perimeter"): Rectangle perimeter calculator.
  - `tabAbout` ("About"): Versioning, copyright, and author contact link.
  - `tabUse` ("Constants"): Mathematical constants ($\pi$, $e$).
  - `tabHelp` ("Assistance"): Embedded tab container for troubleshooting guides.
  - `tabAreas` (" Area"): Circle area and rectangle area calculator.
- **`TabControl1`**: Nested tab control within `tabHelp` containing:
  - `TabPage2` ("Errors"): Error visual display and instructions.
  - `TabPage3` ("Incorrect Answers"): Comprehensive FAQ for troubleshooting unexpected calculation outputs.
- **`tctGame`**: Tab control in the Rock-Paper-Scissors sidecar application hosting:
  - `tabGame` ("Game"): Interactive RPS play area.
  - `tabAbout` ("About"): RPS version and copyright information.

### 2. Button
- **Evaluation Triggers**: `btnEuqals`, `btnEuqlas` ("Evaluate"), `btn1` ("Test" prime check), `btnSqRt` ("Find Square Root"), `btnGo` ("Go" RPS turn), `btnGetNew` ("Generate unique random numbers").
- **Area & Geometry Helpers**: `btnArea` ("Get area" for circle), `btnRectArea` ("Get area" for rectangle), `btnGetperi` ("Get perimeter").
- **Temperature Unit Changers**: `btnFahrenhe` ("Change to Fahrenheit"), `btnCelsius` ("Change to Celsius").
- **Constants**: `btnPi` ("Pi"), `btneconst` ("e").
- **Randomization**: `btnRand` ("Refresh random numbers").
- **Session & Exit Management**: `btnClearAll` ("Clear All"), `btnExti` ("Quit"), `Button1` ("About this software...").

### 3. TextBox
- **Operands & Inputs**: `txtN1`, `txtN2` (First and second operands in basic calculator), `txtRadiusValue` (Circle radius), `txtBase`, `txtHeighth` (Rectangle dimensions), `txtRectPeri1`, `txtRectPeri2` (Rectangle perimeter sides), `txtFahrenhe` (Temperature input), `txtSqRt` (Square root input), `txtNum1` (Prime check input), `txtFn` (Operator symbol input).
- **Scratch Pads & Notes**: `txtSP1`, `TextBox1` through `TextBox16` (Scratch pad text areas on `tabMake` and help screens).

### 4. Label
- **Dynamic Answer Displays**: `lblYourAnswer` (Main answer output string), `lblAns` (Temperature result readout), `lbl1` (Prime verification output), `lblRan1`..`lblRan8` (Random output values), `lblNumber` (Random number generation iteration counter).
- **Static Descriptors & Headers**: `lblCalc` ("Calculator"), `lblVersion` ("Version 4.0"), `lblErr` ("Errors:"), `lblErr1WhatToDo` (Troubleshooting guidance text), `lblAreabyRad`, `lblRect`, `lblFindperi`, `lblThetempe`, `lblEnterate`.
- **RPS Game Counters & Status**: `lblYourDecision`, `lblCompDecision`, `lblWon`, `lblWins`, `lblDraws`, `lblLosses`.

### 5. LinkLabel
- **`lnkWeb`**: Link text `"https://collectivetoolbox.com/ ~ info@collectivetoolbox.com"` displayed on the About tab.

### 6. RadioButton
- **Arithmetic Operators**: `radN` (`+`), `radS` (`-`), `radX` (`*`), `radD` (`/`), `radBk` (`\`), `radMod` (`Mod`), `radE` (`^`), `radAC` (`AC`).
- **Unit Options**: `radQ1` (`a-ft^2`), `radQ9` (`A-ft^2`), `RadioButton3` (`yd^2-ft^2`), `RadioButton4` (`m^2-ft^2`), `RadioButton6` (`rd^2`).
- **RPS Choices**: `radRock` (`rock`), `radPaper` (`paper`), `radScissor` (`scissor`).

### 7. CheckBox
- **`CheckBox1` / `chkLock`**: Checkbox labelled `"Locked?"` / `"Lock session"` to lock calculations and prevent resetting or quitting without unlocking.

### 8. GroupBox
- **`grpChooseOne` / `GroupBox1`**: Container for operator radio buttons.
- **`grpRan`**: Container for random number readouts `lblRan1` through `lblRan8`.
- **`grpQ1` .. `grpQ3`**: Containers for unit selections.

### 9. Panel
- **`Panel1`, `Panel2`, `Panel3`**: Visual borders and background panels enclosing error views and scratch pad regions.

### 10. PictureBox
- **`PictureBox1`, `PictureBox3`**: Display embedded help graphics and error diagrams in `Form3` and `tabHelp`.
- **`pctNew`, `pctQuit`**: Clickable image buttons in RPS for starting a new game session and quitting.

### 11. MainMenu & MenuItem
- **`mnuProgram` / `MainMenu1` / `mmn1` / `mnuRPS`**: Top-level menu bar definitions.
- **`mnuCalc`** ("Calculator"), **`mnuFile`** ("File"), **`mnuNew`** ("New session..."), **`mnuAbout`** ("About..."), **`mnuExit`** ("Exit"), **`MenuItem1`** ("About RPS").
