// SPDX-License-Identifier: AGPL-3.0-or-later AND X11
// SPDX-License-Identifier for parts derived from xkeyboard-config: X11
/*
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

// See full xkeyboard-config license details at end of file.

//! Rebuild generated xkeyboard-config rule outputs inside the staged asset tree.
//!
//! This module ports the installed-data generation logic from these upstream
//! xkeyboard-config helpers and Meson files:
//! - vendor/x11/c_src/xkeyboard-config/xkeyboard-config-2.42/rules/meson.build
//! - vendor/x11/c_src/xkeyboard-config/xkeyboard-config-2.42/rules/merge.py
//! - vendor/x11/c_src/xkeyboard-config/xkeyboard-config-2.42/rules/compat/map-variants.py
//! - vendor/x11/c_src/xkeyboard-config/xkeyboard-config-2.42/rules/generate-options-symbols.py
//! - vendor/x11/c_src/xkeyboard-config/xkeyboard-config-2.42/rules/xml2lst.pl

use anyhow::{Context, Result, bail, ensure};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use xmltree::{Element, XMLNode};

const STATIC_RULE_PARTS: &[&str] = &[
    "0000-hdr.part",
    "0001-lists.part",
    "0002-{ruleset}.lists.part",
    "0004-{ruleset}.model_keycodes.part",
    "0005-layout1_keycodes.part",
    "0006-layout_keycodes.part",
    "0007-options_keycodes.part",
    "0008-modellayout_geometry.part",
    "0009-model_geometry.part",
    "0011-modellayoutvariant_symbols.part",
    "0013-modellayout_symbols.part",
    "0015-modellayout1_symbols.part",
    "0018-modellayout2_symbols.part",
    "0020-modellayout3_symbols.part",
    "0022-modellayout4_symbols.part",
    "0026-{ruleset}.model_symbols.part",
    "0027-{ruleset}.modellayout_symbols1.part",
    "0033-modellayout_compat.part",
    "0034-modellayout1_compat.part",
    "0035-model_types.part",
    "0036-layoutoption_symbols.part",
    "0037-layout1option_symbols.part",
    "0038-layout2option_symbols.part",
    "0039-layout3option_symbols.part",
    "0040-layout4option_symbols.part",
    "compat/0028-layoutvariant_compat.part",
    "compat/0029-layout1variant1_compat.part",
    "compat/0030-layout2variant2_compat.part",
    "compat/0031-layout3variant3_compat.part",
    "compat/0032-layout4variant4_compat.part",
    "compat/0041-option_symbols.part",
];

const GENERATED_RULE_PARTS: &[&str] = &[
    "0042-option_symbols.part",
    "0043-option_compat.part",
    "0044-option_types.part",
    "0010-mlv_s.part",
    "0012-ml_s.part",
    "0014-ml1_s.part",
    "0016-ml1v1_s.part",
    "0017-ml2_s.part",
    "0019-ml3_s.part",
    "0021-ml4_s.part",
    "0023-ml2v2_s.part",
    "0024-ml3v3_s.part",
    "0025-ml4v4_s.part",
];

const SYMBOL_SYMLINK_TARGETS: &[(&str, &str)] = &[
    // Source: vendor/x11/c_src/xkeyboard-config/xkeyboard-config-2.42/meson.build
    ("caps", "capslock"),
    ("esperanto", "epo"),
    ("grp", "group"),
    ("japan", "jp"),
    ("korean", "kr"),
    ("lv2", "level2"),
    ("lv3", "level3"),
    ("lv5", "level5"),
];

const MODELS_HEADER: &str = "! model\n";
const LAYOUTS_HEADER: &str = "\n! layout\n";
const VARIANTS_HEADER: &str = "\n! variant\n";
const OPTIONS_HEADER: &str = "\n! option\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum RulesSection {
    Keycodes,
    Compatibility,
    Geometry,
    Symbols,
    Types,
}

impl RulesSection {
    fn file_dir(self) -> &'static str {
        match self {
            Self::Keycodes => "keycodes",
            Self::Compatibility => "compat",
            Self::Geometry => "geometry",
            Self::Symbols => "symbols",
            Self::Types => "types",
        }
    }

    fn xkb_header_name(self) -> &'static str {
        match self {
            Self::Keycodes => "keycodes",
            Self::Compatibility => "compatibility",
            Self::Geometry => "geometry",
            Self::Symbols => "symbols",
            Self::Types => "types",
        }
    }

    fn rules_name(self) -> &'static str {
        match self {
            Self::Keycodes => "keycodes",
            Self::Compatibility => "compat",
            Self::Geometry => "geometry",
            Self::Symbols => "symbols",
            Self::Types => "types",
        }
    }
}

#[derive(Clone, Debug)]
struct Directive {
    option_name: String,
    filename: String,
    section: String,
}

impl Directive {
    fn render(&self) -> String {
        format!("{}({})", self.filename, self.section)
    }
}

#[derive(Clone, Debug, Default)]
struct DirectiveSet {
    keycodes: Option<Directive>,
    compatibility: Option<Directive>,
    geometry: Option<Directive>,
    symbols: Option<Directive>,
    types: Option<Directive>,
}

impl DirectiveSet {
    fn is_empty(&self) -> bool {
        self.keycodes.is_none()
            && self.compatibility.is_none()
            && self.geometry.is_none()
            && self.symbols.is_none()
            && self.types.is_none()
    }

    fn get(&self, section: RulesSection) -> Option<&Directive> {
        match section {
            RulesSection::Keycodes => self.keycodes.as_ref(),
            RulesSection::Compatibility => self.compatibility.as_ref(),
            RulesSection::Geometry => self.geometry.as_ref(),
            RulesSection::Symbols => self.symbols.as_ref(),
            RulesSection::Types => self.types.as_ref(),
        }
    }

    fn get_mut(&mut self, section: RulesSection) -> &mut Option<Directive> {
        match section {
            RulesSection::Keycodes => &mut self.keycodes,
            RulesSection::Compatibility => &mut self.compatibility,
            RulesSection::Geometry => &mut self.geometry,
            RulesSection::Symbols => &mut self.symbols,
            RulesSection::Types => &mut self.types,
        }
    }
}

#[derive(Clone, Debug)]
struct LayoutSpec {
    layout: String,
    variant: Option<String>,
}

impl LayoutSpec {
    fn parse(layout: &str, variant: Option<&str>) -> Result<Self> {
        if let Some(raw_variant) = variant {
            return Ok(Self {
                layout: layout.to_string(),
                variant: Some(raw_variant.to_string()),
            });
        }

        if let Some((parsed_layout, rest)) = layout.split_once('(') {
            let (parsed_variant, _) =
                rest.split_once(')').with_context(|| {
                    format!("Missing ')' in layout mapping token {layout}")
                })?;
            return Ok(Self {
                layout: parsed_layout.to_string(),
                variant: Some(parsed_variant.to_string()),
            });
        }

        Ok(Self {
            layout: layout.to_string(),
            variant: None,
        })
    }

    fn render(&self) -> String {
        if let Some(variant) = &self.variant {
            format!("{}({variant})", self.layout)
        } else {
            self.layout.clone()
        }
    }
}

#[derive(Clone, Debug)]
struct MappingPair {
    left: LayoutSpec,
    right: LayoutSpec,
}

/// Generate the installed rules outputs inside a staged XKB directory.
pub(crate) fn generate_xkb_rules(xkb_root: &Path) -> Result<()> {
    let rules_dir = xkb_root.join("rules");
    if !rules_dir.is_dir() {
        bail!("Missing rules directory at {}", rules_dir.display());
    }

    generate_option_parts(xkb_root, &rules_dir)?;
    generate_compat_parts(&rules_dir)?;
    generate_ruleset_files(&rules_dir)?;
    generate_xml_and_lst_outputs(&rules_dir)?;

    Ok(())
}

/// Port of `rules/generate-options-symbols.py` from xkeyboard-config.
fn generate_option_parts(xkb_root: &Path, rules_dir: &Path) -> Result<()> {
    let base_xml_path = rules_dir.join("base.xml");
    let base_extras_xml_path = rules_dir.join("base.extras.xml");
    let option_names = collect_option_names(&base_xml_path)?
        .into_iter()
        .chain(collect_option_names(&base_extras_xml_path)?)
        .collect::<BTreeSet<_>>();
    let skip = find_options_to_skip(rules_dir)?;

    for (output, section) in [
        ("0042-option_symbols.part", RulesSection::Symbols),
        ("0043-option_compat.part", RulesSection::Compatibility),
        ("0044-option_types.part", RulesSection::Types),
    ] {
        let mut content = String::new();
        let _ = writeln!(
            content,
            "! option                         = {}",
            section.rules_name()
        );

        for option_name in &option_names {
            if skip.contains(option_name) || option_name.starts_with("custom:")
            {
                continue;
            }

            let directives = resolve_option(xkb_root, option_name)?;
            ensure!(
                !directives.is_empty(),
                "Option {option_name} does not resolve to any XKB section"
            );

            let Some(directive) = directives.get(section) else {
                continue;
            };

            let _ = writeln!(
                content,
                "  {:30} = +{}",
                directive.option_name,
                directive.render()
            );
        }

        if section == RulesSection::Types {
            let _ = writeln!(content, "  {:30} = +custom", "custom:types");
        }

        fs::write(rules_dir.join(output), content).with_context(|| {
            format!("Failed to write generated XKB rules part {output}")
        })?;
    }

    Ok(())
}

/// Port of `rules/compat/map-variants.py` as used by `rules/meson.build`.
fn generate_compat_parts(rules_dir: &Path) -> Result<()> {
    let compat_dir = rules_dir.join("compat");
    let layout_mappings =
        parse_mapping_file(&compat_dir.join("layoutsMapping.lst"))?;
    let variant_mappings =
        parse_mapping_file(&compat_dir.join("variantsMapping.lst"))?;

    for level in 0_u8..=4 {
        let number = if level == 0 { None } else { Some(level) };
        let ml_s_name = match level {
            0 => "0012-ml_s.part",
            1 => "0014-ml1_s.part",
            2 => "0017-ml2_s.part",
            3 => "0019-ml3_s.part",
            4 => "0021-ml4_s.part",
            _ => bail!("Unsupported mapping level {level}"),
        };
        let variant_part_name = match level {
            0 => "0010-mlv_s.part",
            1 => "0016-ml1v1_s.part",
            2 => "0023-ml2v2_s.part",
            3 => "0024-ml3v3_s.part",
            4 => "0025-ml4v4_s.part",
            _ => bail!("Unsupported variant mapping level {level}"),
        };

        let mut ml_s = String::new();
        write_mls(&mut ml_s, &layout_mappings, number, true);
        write_mls(&mut ml_s, &variant_mappings, number, false);
        fs::write(rules_dir.join(ml_s_name), ml_s).with_context(|| {
            format!("Failed to write generated XKB compat part {ml_s_name}")
        })?;

        let mut variant_part = String::new();
        write_mlvs(&mut variant_part, &variant_mappings, number, true)?;
        fs::write(rules_dir.join(variant_part_name), variant_part).with_context(|| {
            format!("Failed to write generated XKB compat part {variant_part_name}")
        })?;
    }

    Ok(())
}

/// Port of `rules/merge.py` as invoked from `rules/meson.build`.
fn generate_ruleset_files(rules_dir: &Path) -> Result<()> {
    for ruleset in ["base", "evdev"] {
        let mut part_paths = Vec::new();
        for template in STATIC_RULE_PARTS {
            let relative = template.replace("{ruleset}", ruleset);
            part_paths.push(rules_dir.join(relative));
        }
        for generated in GENERATED_RULE_PARTS {
            part_paths.push(rules_dir.join(generated));
        }

        let merged = merge_rule_parts(&part_paths)?;
        fs::write(rules_dir.join(ruleset), merged).with_context(|| {
            format!("Failed to write generated XKB rules file {ruleset}")
        })?;
    }

    Ok(())
}

/// Port of the rules XML copy and `xml2lst.pl` list generation steps.
fn generate_xml_and_lst_outputs(rules_dir: &Path) -> Result<()> {
    let base_xml = rules_dir.join("base.xml");
    let base_extras_xml = rules_dir.join("base.extras.xml");
    let evdev_xml = rules_dir.join("evdev.xml");
    let evdev_extras_xml = rules_dir.join("evdev.extras.xml");

    fs::copy(&base_xml, &evdev_xml).with_context(|| {
        format!(
            "Failed to copy {} to {}",
            base_xml.display(),
            evdev_xml.display()
        )
    })?;
    fs::copy(&base_extras_xml, &evdev_extras_xml).with_context(|| {
        format!(
            "Failed to copy {} to {}",
            base_extras_xml.display(),
            evdev_extras_xml.display()
        )
    })?;

    for (xml_name, lst_name) in
        [("base.xml", "base.lst"), ("evdev.xml", "evdev.lst")]
    {
        let lst = xml_to_lst(&rules_dir.join(xml_name))?;
        fs::write(rules_dir.join(lst_name), lst).with_context(|| {
            format!("Failed to write generated XKB list file {lst_name}")
        })?;
    }

    Ok(())
}

fn collect_option_names(xml_path: &Path) -> Result<Vec<String>> {
    let root = parse_xml(xml_path)?;
    let mut names = Vec::new();

    for option_list in children_named(&root, "optionList") {
        for group in children_named(option_list, "group") {
            for option in children_named(group, "option") {
                let config_item = child_named(option, "configItem")
                    .with_context(|| {
                        format!(
                            "Missing configItem in option element in {}",
                            xml_path.display()
                        )
                    })?;
                let name =
                    child_text(config_item, "name").with_context(|| {
                        format!("Missing option name in {}", xml_path.display())
                    })?;
                names.push(name);
            }
        }
    }

    Ok(names)
}

fn find_options_to_skip(rules_dir: &Path) -> Result<BTreeSet<String>> {
    let mut skip = BTreeSet::new();
    for entry in fs::read_dir(rules_dir)
        .with_context(|| format!("Failed to read {}", rules_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("part") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|value| value.to_str())
        else {
            continue;
        };
        if !stem.contains("option") {
            continue;
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let mut option_index: Option<usize> = None;
        for line in content.lines() {
            if line.starts_with("//") || !line.contains('=') {
                continue;
            }

            if line.starts_with('!') {
                if option_index.is_some() || line.contains('$') {
                    continue;
                }

                let tokens =
                    line.split_whitespace().skip(1).collect::<Vec<_>>();
                if let Some(index) =
                    tokens.iter().position(|token| *token == "option")
                {
                    option_index = Some(index);
                }
                continue;
            }

            let Some(index) = option_index else {
                continue;
            };
            let tokens = line.split_whitespace().collect::<Vec<_>>();
            if let Some(token) = tokens.get(index) {
                skip.insert((*token).to_string());
            }
        }
    }

    Ok(skip)
}

fn resolve_option(xkb_root: &Path, option_name: &str) -> Result<DirectiveSet> {
    let Some((filename, section_name)) = option_name.split_once(':') else {
        bail!("Invalid XKB option name {option_name}");
    };

    let mut directives = DirectiveSet::default();
    for section in [
        RulesSection::Keycodes,
        RulesSection::Compatibility,
        RulesSection::Geometry,
        RulesSection::Symbols,
        RulesSection::Types,
    ] {
        let subdir = xkb_root.join(section.file_dir());
        let Some(resolved_name) =
            resolve_section_file(section, &subdir, filename)?
        else {
            continue;
        };

        let section_file = subdir.join(&resolved_name);
        let section_header =
            format!("xkb_{} \"{}\"", section.xkb_header_name(), section_name);
        let content = fs::read_to_string(&section_file).with_context(|| {
            format!("Failed to read {}", section_file.display())
        })?;
        if content.lines().any(|line| line.contains(&section_header)) {
            *directives.get_mut(section) = Some(Directive {
                option_name: option_name.to_string(),
                filename: resolved_name,
                section: section_name.to_string(),
            });
        }
    }

    Ok(directives)
}

fn resolve_section_file(
    section: RulesSection,
    subdir: &Path,
    filename: &str,
) -> Result<Option<String>> {
    if section == RulesSection::Symbols {
        if let Some(target) = symbol_symlink_target(filename) {
            let canonical = subdir.join(target);
            if canonical.exists() {
                return Ok(Some(target.to_string()));
            }
        }
    }

    let direct = subdir.join(filename);
    if direct.exists() {
        return Ok(Some(filename.to_string()));
    }

    for entry in fs::read_dir(subdir)
        .with_context(|| format!("Failed to read {}", subdir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str())
        else {
            continue;
        };
        if !name.ends_with("_vndr") {
            continue;
        }

        let vendor_candidate = path.join(filename);
        if vendor_candidate.exists() {
            return Ok(Some(format!("{name}/{filename}")));
        }
    }

    Ok(None)
}

fn symbol_symlink_target(filename: &str) -> Option<&'static str> {
    SYMBOL_SYMLINK_TARGETS.iter().find_map(|(alias, target)| {
        if *alias == filename {
            Some(*target)
        } else {
            None
        }
    })
}

fn parse_mapping_file(path: &Path) -> Result<Vec<MappingPair>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let mut mappings = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts = trimmed.split_whitespace().take(4).collect::<Vec<_>>();
        let mapping = match parts.as_slice() {
            [left, right] => MappingPair {
                left: LayoutSpec::parse(left, None)?,
                right: LayoutSpec::parse(right, None)?,
            },
            [left_layout, left_variant, right_layout, right_variant] => {
                MappingPair {
                    left: LayoutSpec::parse(left_layout, Some(left_variant))?,
                    right: LayoutSpec::parse(
                        right_layout,
                        Some(right_variant),
                    )?,
                }
            }
            _ => {
                bail!(
                    "Unsupported mapping line in {}: {trimmed}",
                    path.display()
                )
            }
        };
        mappings.push(mapping);
    }

    Ok(mappings)
}

fn write_mls(
    dest: &mut String,
    mappings: &[MappingPair],
    number: Option<u8>,
    write_header: bool,
) {
    if write_header {
        match number {
            None => dest.push_str("! model\t\tlayout\t\t\t\t=\tsymbols\n"),
            Some(value) => {
                let _ =
                    writeln!(dest, "! model\t\tlayout[{value}]\t=\tsymbols");
            }
        }
    }

    for mapping in mappings {
        match number {
            None => {
                let _ = writeln!(
                    dest,
                    "  *\t\t{}\t\t\t=\tpc+{}",
                    mapping.left.render(),
                    mapping.right.render()
                );
            }
            Some(value) => {
                let base = if value == 1 { "pc" } else { "" };
                let suffix = if value == 1 {
                    String::new()
                } else {
                    format!(":{value}")
                };
                let second_layout = if mapping.right.variant.is_some() {
                    mapping.right.render()
                } else {
                    format!("{}%(v[{value}])", mapping.right.layout)
                };
                let _ = writeln!(
                    dest,
                    "  *\t\t{}\t\t=\t{}+{}{}",
                    mapping.left.render(),
                    base,
                    second_layout,
                    suffix
                );
            }
        }
    }
}

fn write_mlvs(
    dest: &mut String,
    mappings: &[MappingPair],
    number: Option<u8>,
    write_header: bool,
) -> Result<()> {
    if write_header {
        match number {
            None => {
                dest.push_str("! model\t\tlayout\t\tvariant\t\t=\tsymbols\n");
            }
            Some(value) => {
                let _ = writeln!(
                    dest,
                    "! model\t\tlayout[{value}]\tvariant[{value}]\t=\tsymbols"
                );
            }
        }
    }

    for mapping in mappings {
        let Some(left_variant) = &mapping.left.variant else {
            bail!(
                "Variant mapping missing source variant for {}",
                mapping.left.layout
            );
        };
        match number {
            None => {
                let _ = writeln!(
                    dest,
                    "  *\t\t{}\t\t{}\t\t=\tpc+{}",
                    mapping.left.layout,
                    left_variant,
                    mapping.right.render()
                );
            }
            Some(value) => {
                let base = if value == 1 { "pc" } else { "" };
                let suffix = if value == 1 {
                    String::new()
                } else {
                    format!(":{value}")
                };
                let second_layout = if mapping.right.variant.is_some() {
                    mapping.right.render()
                } else {
                    format!("{}%(v[{value}])", mapping.right.layout)
                };
                let _ = writeln!(
                    dest,
                    "  *\t\t{}\t\t{}\t=\t{}+{}{}",
                    mapping.left.layout,
                    left_variant,
                    base,
                    second_layout,
                    suffix
                );
            }
        }
    }

    Ok(())
}

fn merge_rule_parts(files: &[PathBuf]) -> Result<String> {
    let mut sorted = files.to_vec();
    sorted.sort_by(|left, right| left.file_name().cmp(&right.file_name()));

    let mut sections = BTreeMap::<String, Vec<PathBuf>>::new();
    let mut section_order = vec![String::new()];
    sections.insert(String::new(), Vec::new());

    for path in sorted {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let mut lines = content.lines();
        let header = if let Some(first_line) = lines.next() {
            if first_line.starts_with("! ") {
                format!("{first_line}\n")
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let entry = sections.entry(header.clone()).or_default();
        entry.push(path.clone());
        if !section_order.iter().any(|seen| seen == &header) {
            section_order.push(header);
        }
    }

    let mut merged = String::from(
        "// DO NOT EDIT THIS FILE - IT WAS AUTOGENERATED BY ctoolbox asset_packer FROM rules/*.part\n//\n",
    );
    for header in section_order {
        if !header.is_empty() {
            merged.push('\n');
            merged.push_str(&header);
        }
        let Some(paths) = sections.get(&header) else {
            continue;
        };
        for path in paths {
            let content = fs::read_to_string(path).with_context(|| {
                format!("Failed to read {}", path.display())
            })?;
            if header.is_empty() {
                merged.push_str(&content);
                continue;
            }

            let body = if let Some((_, body)) = content.split_once('\n') {
                body
            } else {
                ""
            };
            merged.push_str(body);
        }
    }

    Ok(merged)
}

fn xml_to_lst(xml_path: &Path) -> Result<String> {
    let root = parse_xml(xml_path)?;
    let registry = if root.name == "xkbConfigRegistry" {
        &root
    } else {
        bail!(
            "Unexpected XML root {} in {}",
            root.name,
            xml_path.display()
        );
    };

    let mut output = String::new();
    output.push_str(MODELS_HEADER);
    for model_list in children_named(registry, "modelList") {
        for model in children_named(model_list, "model") {
            let config_item =
                child_named(model, "configItem").with_context(|| {
                    format!(
                        "Missing model configItem in {}",
                        xml_path.display()
                    )
                })?;
            let name = escape_xml_text(&child_text(config_item, "name")?);
            let description =
                escape_xml_text(&child_text(config_item, "description")?);
            let _ = writeln!(output, "  {name:<15} {description}");
        }
    }

    output.push_str(LAYOUTS_HEADER);
    for layout_list in children_named(registry, "layoutList") {
        for layout in children_named(layout_list, "layout") {
            let config_item =
                child_named(layout, "configItem").with_context(|| {
                    format!(
                        "Missing layout configItem in {}",
                        xml_path.display()
                    )
                })?;
            let name = escape_xml_text(&child_text(config_item, "name")?);
            let description =
                escape_xml_text(&child_text(config_item, "description")?);
            let _ = writeln!(output, "  {name:<15} {description}");
        }
    }

    output.push_str(VARIANTS_HEADER);
    for layout_list in children_named(registry, "layoutList") {
        for layout in children_named(layout_list, "layout") {
            let layout_config = child_named(layout, "configItem")
                .with_context(|| {
                    format!(
                        "Missing layout configItem in {}",
                        xml_path.display()
                    )
                })?;
            let layout_name =
                escape_xml_text(&child_text(layout_config, "name")?);
            for variant_list in children_named(layout, "variantList") {
                for variant in children_named(variant_list, "variant") {
                    let config_item = child_named(variant, "configItem")
                        .with_context(|| {
                            format!(
                                "Missing variant configItem in {}",
                                xml_path.display()
                            )
                        })?;
                    let name =
                        escape_xml_text(&child_text(config_item, "name")?);
                    let description = escape_xml_text(&child_text(
                        config_item,
                        "description",
                    )?);
                    let _ = writeln!(
                        output,
                        "  {name:<15} {layout_name}: {description}"
                    );
                }
            }
        }
    }

    output.push_str(OPTIONS_HEADER);
    for option_list in children_named(registry, "optionList") {
        for group in children_named(option_list, "group") {
            let group_config =
                child_named(group, "configItem").with_context(|| {
                    format!(
                        "Missing option group configItem in {}",
                        xml_path.display()
                    )
                })?;
            let group_name =
                escape_xml_text(&child_text(group_config, "name")?);
            let group_description =
                escape_xml_text(&child_text(group_config, "description")?);
            let _ = writeln!(output, "  {group_name:<20} {group_description}");

            for option in children_named(group, "option") {
                let option_config = child_named(option, "configItem")
                    .with_context(|| {
                        format!(
                            "Missing option configItem in {}",
                            xml_path.display()
                        )
                    })?;
                let name = escape_xml_text(&child_text(option_config, "name")?);
                let description =
                    escape_xml_text(&child_text(option_config, "description")?);
                let _ = writeln!(output, "  {name:<20} {description}");
            }
        }
    }

    Ok(output)
}

fn parse_xml(path: &Path) -> Result<Element> {
    let file = fs::File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    Element::parse(file)
        .with_context(|| format!("Failed to parse XML {}", path.display()))
}

fn children_named<'a>(
    element: &'a Element,
    name: &'a str,
) -> impl Iterator<Item = &'a Element> {
    element
        .children
        .iter()
        .filter_map(move |child| match child {
            XMLNode::Element(child_element) if child_element.name == name => {
                Some(child_element)
            }
            _ => None,
        })
}

fn child_named<'a>(element: &'a Element, name: &'a str) -> Option<&'a Element> {
    children_named(element, name).next()
}

fn child_text(element: &Element, name: &str) -> Result<String> {
    let child = child_named(element, name).with_context(|| {
        format!("Missing XML child {name} under {}", element.name)
    })?;
    let Some(text) = child.get_text() else {
        bail!("Missing XML text for {name} under {}", element.name);
    };
    Ok(text.into_owned())
}

fn escape_xml_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/*

// This file is based on xkeyboard-config-2.42, used per the terms below.



AUTHORS of xkeyboard-config-2.42:



== Initiator and maintainer:
Сергей Удальцов (Sergey Udaltsov) <svu@users.sourceforge.net>

== Major contributions by:
Andriy Rysin <arysin@myrealbox.com>
Denis Barbier <barbier@linuxfr.org>
Frank Murphy <murphyf+xfree86@f-m.fm>
Ivan Pascal <pascal@info.tsu.ru>
Nicolas Mailhot <nicolas.mailhot@laposte.net>
Данило Шеган <dsegan@gmx.net>

== Substantial contributions by:
Ivan A Derzhanski <iad@math.bas.bg>
Runa Aruna <runa_aruna@yahoo.com>
Frédéric BOITEUX <fboiteux@calistel.com>




COPYING:



Copyright 1996 by Joseph Moss
Copyright (C) 2002-2007 Free Software Foundation, Inc.
Copyright (C) Dmitry Golubev <lastguru@mail.ru>, 2003-2004
Copyright (C) 2004, Gregory Mokhin <mokhin@bog.msu.ru>
Copyright (C) 2006 Erdal Ronahî

Permission to use, copy, modify, distribute, and sell this software and its
documentation for any purpose is hereby granted without fee, provided that
the above copyright notice appear in all copies and that both that
copyright notice and this permission notice appear in supporting
documentation, and that the name of the copyright holder(s) not be used in
advertising or publicity pertaining to distribution of the software without
specific, written prior permission.  The copyright holder(s) makes no
representations about the suitability of this software for any purpose.  It
is provided "as is" without express or implied warranty.

THE COPYRIGHT HOLDER(S) DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE,
INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS, IN NO
EVENT SHALL THE COPYRIGHT HOLDER(S) BE LIABLE FOR ANY SPECIAL, INDIRECT OR
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE,
DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER
TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.


Copyright (c) 1996  Digital Equipment Corporation

Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be included
in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL DIGITAL EQUIPMENT CORPORATION BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR
THE USE OR OTHER DEALINGS IN THE SOFTWARE.

Except as contained in this notice, the name of the Digital Equipment
Corporation shall not be used in advertising or otherwise to promote
the sale, use or other dealings in this Software without prior written
authorization from Digital Equipment Corporation.


Copyright 1996, 1998  The Open Group

Permission to use, copy, modify, distribute, and sell this software and its
documentation for any purpose is hereby granted without fee, provided that
the above copyright notice appear in all copies and that both that
copyright notice and this permission notice appear in supporting
documentation.

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE OPEN GROUP BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

Except as contained in this notice, the name of The Open Group shall
not be used in advertising or otherwise to promote the sale, use or
other dealings in this Software without prior written authorization
from The Open Group.


Copyright 2004-2005 Sun Microsystems, Inc.  All rights reserved.

Permission is hereby granted, free of charge, to any person obtaining a
copy of this software and associated documentation files (the "Software"),
to deal in the Software without restriction, including without limitation
the rights to use, copy, modify, merge, publish, distribute, sublicense,
and/or sell copies of the Software, and to permit persons to whom the
Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice (including the next
paragraph) shall be included in all copies or substantial portions of the
Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.


Copyright (c) 1996 by Silicon Graphics Computer Systems, Inc.

Permission to use, copy, modify, and distribute this
software and its documentation for any purpose and without
fee is hereby granted, provided that the above copyright
notice appear in all copies and that both that copyright
notice and this permission notice appear in supporting
documentation, and that the name of Silicon Graphics not be
used in advertising or publicity pertaining to distribution
of the software without specific prior written permission.
Silicon Graphics makes no representation about the suitability
of this software for any purpose. It is provided "as is"
without any express or implied warranty.

SILICON GRAPHICS DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS
SOFTWARE, INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS FOR A PARTICULAR PURPOSE. IN NO EVENT SHALL SILICON
GRAPHICS BE LIABLE FOR ANY SPECIAL, INDIRECT OR CONSEQUENTIAL
DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE,
DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE
OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION  WITH
THE USE OR PERFORMANCE OF THIS SOFTWARE.


Copyright (c) 1996  X Consortium

Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE X CONSORTIUM BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

Except as contained in this notice, the name of the X Consortium shall
not be used in advertising or otherwise to promote the sale, use or
other dealings in this Software without prior written authorization
from the X Consortium.


Copyright (C) 2004, 2006 Ævar Arnfjörð Bjarmason <avarab@gmail.com>

Permission to use, copy, modify, distribute, and sell this software and its
documentation for any purpose is hereby granted without fee, provided that
the above copyright notice appear in all copies and that both that
copyright notice and this permission notice appear in supporting
documentation.

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE OPEN GROUP BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

Except as contained in this notice, the name of a copyright holder shall
not be used in advertising or otherwise to promote the sale, use or
other dealings in this Software without prior written authorization of
the copyright holder.


Copyright (C) 1999, 2000 by Anton Zinoviev <anton@lml.bas.bg>

This software may be used, modified, copied, distributed, and sold,
in both source and binary form provided that the above copyright
and these terms are retained. Under no circumstances is the author
responsible for the proper functioning of this software, nor does
the author assume any responsibility for damages incurred with its
use.

Permission is granted to anyone to use, distribute and modify
this file in any way, provided that the above copyright notice
is left intact and the author of the modification summarizes
the changes in this header.

This file is distributed without any expressed or implied warranty.

*/
