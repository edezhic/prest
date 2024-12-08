// todo: add daisyui classes
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
struct GradientStop {
    color: String,
    position: String,
}

lazy_static! {
    static ref CONTAINER_STYLES: HashMap<&'static str, (&'static str, &'static str)> = {
        let mut m = HashMap::new();
        m.insert("sm", ("640px", "640px"));
        m.insert("md", ("768px", "768px"));
        m.insert("lg", ("1024px", "1024px"));
        m.insert("xl", ("1280px", "1280px"));
        m.insert("2xl", ("1536px", "1536px"));
        m
    };
    static ref COLOR_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("black", "#000000");
        m.insert("white", "#ffffff");
        m.insert("gray", "#6b7280");
        m.insert("red", "#ef4444");
        m.insert("yellow", "#eab308");
        m.insert("green", "#22c55e");
        m.insert("blue", "#3b82f6");
        m.insert("indigo", "#6366f1");
        m.insert("purple", "#a855f7");
        m.insert("pink", "#ec4899");
        m
    };
}

pub fn compile(input: &str, element_id: &str) -> (Option<String>, Option<String>) {
    let classes: Vec<&str> = input.split_whitespace().collect();
    let mut compiled_styles: HashMap<String, Vec<String>> = HashMap::new();
    let mut inline_styles = Vec::new();
    let mut container_sizes = HashSet::new();
    let mut gradient_stops: Vec<GradientStop> = Vec::new();
    let mut gradient_direction = "to right".to_string();

    for class in classes {
        if let Some(color_stop) = parse_gradient_color_stop(class) {
            gradient_stops.push(color_stop);
        } else if class.starts_with("bg-gradient-to-") {
            if let Some(direction) = parse_background(class) {
                gradient_direction = direction
                    .split('(')
                    .nth(1)
                    .and_then(|s| s.split(',').next())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "to right".to_string());
            }
        } else if let Some((variants, style)) = parse_class(class) {
            if variants.is_empty() {
                inline_styles.push(style);
            } else {
                let key = variants.join(":");
                compiled_styles
                    .entry(key)
                    .or_insert_with(Vec::new)
                    .push(style);
            }
        } else if class == "container" {
            container_sizes.insert("default");
        } else if class.starts_with("container-") {
            container_sizes.insert(&class[10..]);
        } else {
            panic!("Unrecognized class: {}", class);
        }
    }

    // If gradient stops are present, update the background-image style
    if !gradient_stops.is_empty() {
        let gradient_style = format!(
            "background-image: linear-gradient({}, {});",
            gradient_direction,
            gradient_stops
                .iter()
                .map(|stop| format!("{} {}", stop.color, stop.position))
                .collect::<Vec<_>>()
                .join(", ")
        );
        inline_styles.push(gradient_style);
    }

    let mut inline_styles = if inline_styles.is_empty() {
        None
    } else {
        Some(resolve_conflicts(inline_styles).join(" "))
    };

    let mut complex_styles = if compiled_styles.is_empty() && container_sizes.is_empty() {
        None
    } else {
        let mut styles = nest_styles(compiled_styles, element_id);
        if !container_sizes.is_empty() {
            styles.push_str(&generate_container_styles(element_id, &container_sizes));
        }
        Some(styles)
    };

    if let Some(ref mut styles) = complex_styles {
        if let Some(inline) = inline_styles {
            // Merge styles to avoid inline ones overriding complex ones
            *styles = format!("#{element_id} {{ {inline} }} {styles}");
            inline_styles = None;
        }
    }

    (inline_styles, complex_styles)
}

fn generate_container_styles(element_id: &str, sizes: &HashSet<&str>) -> String {
    let mut styles = format!("#{} {{ width: 100%; }}\n", element_id);

    let mut sorted_sizes: Vec<&&str> = sizes.iter().collect();
    sorted_sizes.sort_by(|a, b| {
        let a_val = CONTAINER_STYLES
            .get(**a)
            .map(|(min_width, _)| min_width)
            .unwrap_or(&"0px");
        let b_val = CONTAINER_STYLES
            .get(**b)
            .map(|(min_width, _)| min_width)
            .unwrap_or(&"0px");
        a_val.cmp(b_val)
    });

    for &size in sorted_sizes {
        if size == "default" {
            for (_breakpoint, (min_width, max_width)) in CONTAINER_STYLES.iter() {
                styles.push_str(&format!(
                    "@media (min-width: {}) {{ #{} {{ max-width: {}; }} }}\n",
                    min_width, element_id, max_width
                ));
            }
        } else if let Some((min_width, max_width)) = CONTAINER_STYLES.get(size) {
            styles.push_str(&format!(
                "@media (min-width: {}) {{ #{} {{ max-width: {}; }} }}\n",
                min_width, element_id, max_width
            ));
        }
    }

    styles
}

fn parse_class(class: &str) -> Option<(Vec<String>, String)> {
    if class.starts_with('[') && class.ends_with(']') {
        Some((vec![], format!("{class};")))
    } else {
        let parts: Vec<&str> = class.split(':').collect();
        let base_class = parts.last()?;
        let variants = parts[..parts.len() - 1].to_vec();
        let style = parse_base_class(base_class)?;
        Some((variants.into_iter().map(String::from).collect(), style))
    }
}

fn parse_base_class(class: &str) -> Option<String> {
    let parsers: Vec<fn(&str) -> Option<String>> = vec![
        parse_layout,
        parse_svg_props,
        parse_spacing,
        parse_size,
        parse_color,
        parse_display,
        parse_text,
        parse_typography,
        parse_flex,
        parse_grid,
        parse_alignment,
        parse_font,
        parse_background,
        parse_border,
        parse_border_radius,
        parse_border_spacing,
        parse_shadow,
        parse_opacity,
        parse_z_index,
        parse_max_width,
        parse_height,
        parse_min_height,
        parse_max_height,
        parse_animation,
        parse_gap,
        parse_transition,
        parse_transform,
        parse_filter,
        parse_backdrop_filter,
        parse_interactivity,
        parse_max_width,
        parse_min_width,
        parse_height,
        parse_min_height,
        parse_max_height,
        parse_font_size,
        parse_font_weight,
        parse_letter_spacing,
        parse_line_height,
        parse_list_style_type,
        parse_list_style_position,
        parse_placeholder_color,
        parse_placeholder_opacity,
        parse_text_align,
        parse_text_decoration,
        parse_text_transform,
        parse_vertical_align,
        parse_whitespace,
        parse_word_break,
    ];

    for parser in parsers {
        if let Some(style) = parser(class) {
            return Some(style);
        }
    }

    parse_arbitrary_value(class)
}

fn parse_value(value: &str, predefined: &[(&str, &str)]) -> Option<String> {
    if value == "auto" {
        Some("auto".to_owned())
    } else if value.starts_with('[') && value.ends_with(']') {
        Some(value[1..value.len() - 1].to_string())
    } else {
        predefined
            .iter()
            .find(|&&(k, _)| k == value)
            .map(|&(_, v)| v.to_string())
    }
}

fn parse_property(
    class: &str,
    prefix: &str,
    property: &str,
    values: &[(&str, &str)],
) -> Option<String> {
    let mut without_prefix = class.strip_prefix(prefix);
    if let Some(rest) = without_prefix {
        if rest.starts_with('-') {
            without_prefix = class.strip_prefix(prefix).unwrap().strip_prefix('-')
        }
    }
    without_prefix
        .and_then(|value| parse_value(value, values).map(|v| format!("{}: {};", property, v)))
}

fn parse_height(class: &str) -> Option<String> {
    let height_values = [
        ("h-auto", "auto"),
        ("h-full", "100%"),
        ("h-screen", "100vh"),
        ("h-min", "min-content"),
        ("h-max", "max-content"),
        ("h-fit", "fit-content"),
    ];

    parse_property(class, "", "height", &height_values).or_else(|| parse_size(class))
}

fn parse_min_height(class: &str) -> Option<String> {
    let min_height_values = [
        ("min-h-0", "0px"),
        ("min-h-full", "100%"),
        ("min-h-screen", "100vh"),
        ("min-h-min", "min-content"),
        ("min-h-max", "max-content"),
        ("min-h-fit", "fit-content"),
    ];

    parse_property(class, "", "min-height", &min_height_values)
}

fn parse_max_height(class: &str) -> Option<String> {
    let max_height_values = [
        ("max-h-full", "100%"),
        ("max-h-screen", "100vh"),
        ("max-h-min", "min-content"),
        ("max-h-max", "max-content"),
        ("max-h-fit", "fit-content"),
    ];

    parse_property(class, "", "max-height", &max_height_values).or_else(|| {
        if class.starts_with("max-h-") {
            parse_size(class).map(|v| v.replace("width", "max-height"))
        } else {
            None
        }
    })
}

fn parse_max_width(class: &str) -> Option<String> {
    let max_width_values = [
        ("max-w-0", "0rem"),
        ("max-w-none", "none"),
        ("max-w-xs", "20rem"),
        ("max-w-sm", "24rem"),
        ("max-w-md", "28rem"),
        ("max-w-lg", "32rem"),
        ("max-w-xl", "36rem"),
        ("max-w-2xl", "42rem"),
        ("max-w-3xl", "48rem"),
        ("max-w-4xl", "56rem"),
        ("max-w-5xl", "64rem"),
        ("max-w-6xl", "72rem"),
        ("max-w-7xl", "80rem"),
        ("max-w-full", "100%"),
        ("max-w-min", "min-content"),
        ("max-w-max", "max-content"),
        ("max-w-fit", "fit-content"),
        ("max-w-prose", "65ch"),
        ("max-w-screen-sm", "640px"),
        ("max-w-screen-md", "768px"),
        ("max-w-screen-lg", "1024px"),
        ("max-w-screen-xl", "1280px"),
        ("max-w-screen-2xl", "1536px"),
    ];

    parse_property(class, "", "max-width", &max_width_values)
}

fn parse_min_width(class: &str) -> Option<String> {
    let min_width_values = [
        ("min-w-0", "0px"),
        ("min-w-full", "100%"),
        ("min-w-min", "min-content"),
        ("min-w-max", "max-content"),
        ("min-w-fit", "fit-content"),
    ];

    parse_property(class, "", "min-width", &min_width_values)
}

fn parse_border_spacing(class: &str) -> Option<String> {
    if class.starts_with("border-spacing-") {
        let value = class.strip_prefix("border-spacing-")?;
        let spacing = parse_spacing(&format!("p-{}", value))?;
        return Some(spacing.replace("padding", "border-spacing"));
    }
    None
}

fn parse_svg_props(class: &str) -> Option<String> {
    let svg_props = [
        ("fill", "fill"),
        ("stroke", "stroke"),
        ("stroke-width", "stroke-width"),
    ];

    for (prefix, property) in &svg_props {
        if class.starts_with(prefix) {
            let value = class.strip_prefix(prefix).unwrap();
            if let Some(color) = parse_color(&format!("text-{}", value)) {
                return Some(color.replace("color", property));
            } else if value.starts_with('[') && value.ends_with(']') {
                return Some(format!("{}: {};", property, &value[1..value.len() - 1]));
            }
        }
    }
    None
}

fn parse_interactivity(class: &str) -> Option<String> {
    let interactivity_values = [
        ("cursor-auto", "cursor: auto;"),
        ("cursor-default", "cursor: default;"),
        ("cursor-pointer", "cursor: pointer;"),
        ("cursor-wait", "cursor: wait;"),
        ("cursor-text", "cursor: text;"),
        ("cursor-move", "cursor: move;"),
        ("cursor-help", "cursor: help;"),
        ("cursor-not-allowed", "cursor: not-allowed;"),
        ("select-none", "user-select: none;"),
        ("select-text", "user-select: text;"),
        ("select-all", "user-select: all;"),
        ("select-auto", "user-select: auto;"),
        ("resize-none", "resize: none;"),
        ("resize", "resize: both;"),
        ("resize-y", "resize: vertical;"),
        ("resize-x", "resize: horizontal;"),
        ("pointer-events-none", "pointer-events: none;"),
        ("pointer-events-auto", "pointer-events: auto;"),
    ];

    interactivity_values
        .iter()
        .find(|&&(k, _)| k == class)
        .map(|&(_, v)| v.to_string())
}

fn parse_spacing(class: &str) -> Option<String> {
    let spacing_values = [
        ("0", "0"),
        ("px", "1px"),
        ("0.5", "0.125rem"),
        ("1", "0.25rem"),
        ("1.5", "0.375rem"),
        ("2", "0.5rem"),
        ("2.5", "0.625rem"),
        ("3", "0.75rem"),
        ("3.5", "0.875rem"),
        ("4", "1rem"),
        ("5", "1.25rem"),
        ("6", "1.5rem"),
        ("7", "1.75rem"),
        ("8", "2rem"),
        ("9", "2.25rem"),
        ("10", "2.5rem"),
        ("11", "2.75rem"),
        ("12", "3rem"),
        ("14", "3.5rem"),
        ("16", "4rem"),
        ("20", "5rem"),
        ("24", "6rem"),
        ("28", "7rem"),
        ("32", "8rem"),
        ("36", "9rem"),
        ("40", "10rem"),
        ("44", "11rem"),
        ("48", "12rem"),
        ("52", "13rem"),
        ("56", "14rem"),
        ("60", "15rem"),
        ("64", "16rem"),
        ("72", "18rem"),
        ("80", "20rem"),
        ("96", "24rem"),
    ];

    let (property, value) = class.split_once('-')?;
    let prop = match property {
        "p" => "padding".to_owned(),
        "m" => "margin".to_owned(),
        "pt" | "mt" => format!(
            "{}-top",
            if property.starts_with('p') {
                "padding"
            } else {
                "margin"
            }
        ),
        "pr" | "mr" => format!(
            "{}-right",
            if property.starts_with('p') {
                "padding"
            } else {
                "margin"
            }
        ),
        "pb" | "mb" => format!(
            "{}-bottom",
            if property.starts_with('p') {
                "padding"
            } else {
                "margin"
            }
        ),
        "pl" | "ml" => format!(
            "{}-left",
            if property.starts_with('p') {
                "padding"
            } else {
                "margin"
            }
        ),
        "px" | "mx" => {
            return Some(format!(
                "{0}-left: {1}; {0}-right: {1};",
                if property.starts_with('p') {
                    "padding"
                } else {
                    "margin"
                },
                parse_value(value, &spacing_values)?
            ))
        }
        "py" | "my" => {
            return Some(format!(
                "{0}-top: {1}; {0}-bottom: {1};",
                if property.starts_with('p') {
                    "padding"
                } else {
                    "margin"
                },
                parse_value(value, &spacing_values)?
            ))
        }
        _ => return None,
    };

    Some(format!(
        "{}: {};",
        prop,
        parse_value(value, &spacing_values)?
    ))
}

fn parse_size(class: &str) -> Option<String> {
    let size_values = [
        ("0", "0"),
        ("px", "1px"),
        ("0.5", "0.125rem"),
        ("1", "0.25rem"),
        ("1.5", "0.375rem"),
        ("2", "0.5rem"),
        ("2.5", "0.625rem"),
        ("3", "0.75rem"),
        ("3.5", "0.875rem"),
        ("4", "1rem"),
        ("5", "1.25rem"),
        ("6", "1.5rem"),
        ("7", "1.75rem"),
        ("8", "2rem"),
        ("9", "2.25rem"),
        ("10", "2.5rem"),
        ("11", "2.75rem"),
        ("12", "3rem"),
        ("14", "3.5rem"),
        ("16", "4rem"),
        ("20", "5rem"),
        ("24", "6rem"),
        ("28", "7rem"),
        ("32", "8rem"),
        ("36", "9rem"),
        ("40", "10rem"),
        ("44", "11rem"),
        ("48", "12rem"),
        ("52", "13rem"),
        ("56", "14rem"),
        ("60", "15rem"),
        ("64", "16rem"),
        ("72", "18rem"),
        ("80", "20rem"),
        ("96", "24rem"),
        ("auto", "auto"),
        ("1/2", "50%"),
        ("1/3", "33.333333%"),
        ("2/3", "66.666667%"),
        ("1/4", "25%"),
        ("2/4", "50%"),
        ("3/4", "75%"),
        ("full", "100%"),
        ("screen", "100vh"),
        ("min", "min-content"),
        ("max", "max-content"),
        ("fit", "fit-content"),
    ];

    let prefixes = [
        ("w-", "width"),
        ("h-", "height"),
        ("min-w-", "min-width"),
        ("min-h-", "min-height"),
        ("max-w-", "max-width"),
        ("max-h-", "max-height"),
    ];

    for (prefix, property) in prefixes.iter() {
        if let Some(style) = parse_property(class, prefix, property, &size_values) {
            return Some(style);
        }
    }

    None
}

fn parse_color(class: &str) -> Option<String> {
    let (property, color_class) = class.split_once('-')?;

    let property = match property {
        "text" => {
            if size_like(color_class) {
                return None
            } else {
                "color"
            }
        }
        "bg" => "background-color",
        "border" => "border-color",
        "accent" => "accent-color",
        _ => return None,
    };

    // Check if it's a basic color in COLOR_MAP
    if let Some(&hex) = COLOR_MAP.get(color_class) {
        return Some(format!("{}: {};", property, hex));
    }

    // If not in COLOR_MAP, try to parse as a complex color
    let (color, shade) = color_class.rsplit_once('-')?;
    let hex = get_color_hex(color, shade)?;

    Some(format!("{}: {};", property, hex))
}

fn get_color_hex(color: &str, shade: &str) -> Option<String> {
    let colors = [
        (
            "slate",
            [
                "#f8fafc", "#f1f5f9", "#e2e8f0", "#cbd5e1", "#94a3b8", "#64748b", "#475569",
                "#334155", "#1e293b", "#0f172a", "#020617",
            ],
        ),
        (
            "gray",
            [
                "#f9fafb", "#f3f4f6", "#e5e7eb", "#d1d5db", "#9ca3af", "#6b7280", "#4b5563",
                "#374151", "#1f2937", "#111827", "#030712",
            ],
        ),
        (
            "zinc",
            [
                "#fafafa", "#f4f4f5", "#e4e4e7", "#d4d4d8", "#a1a1aa", "#71717a", "#52525b",
                "#3f3f46", "#27272a", "#18181b", "#09090b",
            ],
        ),
        (
            "neutral",
            [
                "#fafafa", "#f5f5f5", "#e5e5e5", "#d4d4d4", "#a3a3a3", "#737373", "#525252",
                "#404040", "#262626", "#171717", "#0a0a0a",
            ],
        ),
        (
            "stone",
            [
                "#fafaf9", "#f5f5f4", "#e7e5e4", "#d6d3d1", "#a8a29e", "#78716c", "#57534e",
                "#44403c", "#292524", "#1c1917", "#0c0a09",
            ],
        ),
        (
            "red",
            [
                "#fef2f2", "#fee2e2", "#fecaca", "#fca5a5", "#f87171", "#ef4444", "#dc2626",
                "#b91c1c", "#991b1b", "#7f1d1d", "#450a0a",
            ],
        ),
        (
            "orange",
            [
                "#fff7ed", "#ffedd5", "#fed7aa", "#fdba74", "#fb923c", "#f97316", "#ea580c",
                "#c2410c", "#9a3412", "#7c2d12", "#431407",
            ],
        ),
        (
            "amber",
            [
                "#fffbeb", "#fef3c7", "#fde68a", "#fcd34d", "#fbbf24", "#f59e0b", "#d97706",
                "#b45309", "#92400e", "#78350f", "#451a03",
            ],
        ),
        (
            "yellow",
            [
                "#fefce8", "#fef9c3", "#fef08a", "#fde047", "#facc15", "#eab308", "#ca8a04",
                "#a16207", "#854d0e", "#713f12", "#422006",
            ],
        ),
        (
            "lime",
            [
                "#f7fee7", "#ecfccb", "#d9f99d", "#bef264", "#a3e635", "#84cc16", "#65a30d",
                "#4d7c0f", "#3f6212", "#365314", "#1a2e05",
            ],
        ),
        (
            "green",
            [
                "#f0fdf4", "#dcfce7", "#bbf7d0", "#86efac", "#4ade80", "#22c55e", "#16a34a",
                "#15803d", "#166534", "#14532d", "#052e16",
            ],
        ),
        (
            "emerald",
            [
                "#ecfdf5", "#d1fae5", "#a7f3d0", "#6ee7b7", "#34d399", "#10b981", "#059669",
                "#047857", "#065f46", "#064e3b", "#022c22",
            ],
        ),
        (
            "teal",
            [
                "#f0fdfa", "#ccfbf1", "#99f6e4", "#5eead4", "#2dd4bf", "#14b8a6", "#0d9488",
                "#0f766e", "#115e59", "#134e4a", "#042f2e",
            ],
        ),
        (
            "cyan",
            [
                "#ecfeff", "#cffafe", "#a5f3fc", "#67e8f9", "#22d3ee", "#06b6d4", "#0891b2",
                "#0e7490", "#155e75", "#164e63", "#083344",
            ],
        ),
        (
            "sky",
            [
                "#f0f9ff", "#e0f2fe", "#bae6fd", "#7dd3fc", "#38bdf8", "#0ea5e9", "#0284c7",
                "#0369a1", "#075985", "#0c4a6e", "#082f49",
            ],
        ),
        (
            "blue",
            [
                "#eff6ff", "#dbeafe", "#bfdbfe", "#93c5fd", "#60a5fa", "#3b82f6", "#2563eb",
                "#1d4ed8", "#1e40af", "#1e3a8a", "#172554",
            ],
        ),
        (
            "indigo",
            [
                "#eef2ff", "#e0e7ff", "#c7d2fe", "#a5b4fc", "#818cf8", "#6366f1", "#4f46e5",
                "#4338ca", "#3730a3", "#312e81", "#1e1b4b",
            ],
        ),
        (
            "violet",
            [
                "#f5f3ff", "#ede9fe", "#ddd6fe", "#c4b5fd", "#a78bfa", "#8b5cf6", "#7c3aed",
                "#6d28d9", "#5b21b6", "#4c1d95", "#2e1065",
            ],
        ),
        (
            "purple",
            [
                "#faf5ff", "#f3e8ff", "#e9d5ff", "#d8b4fe", "#c084fc", "#a855f7", "#9333ea",
                "#7e22ce", "#6b21a8", "#581c87", "#3b0764",
            ],
        ),
        (
            "fuchsia",
            [
                "#fdf4ff", "#fae8ff", "#f5d0fe", "#f0abfc", "#e879f9", "#d946ef", "#c026d3",
                "#a21caf", "#86198f", "#701a75", "#4a044e",
            ],
        ),
        (
            "pink",
            [
                "#fdf2f8", "#fce7f3", "#fbcfe8", "#f9a8d4", "#f472b6", "#ec4899", "#db2777",
                "#be185d", "#9d174d", "#831843", "#500724",
            ],
        ),
        (
            "rose",
            [
                "#fff1f2", "#ffe4e6", "#fecdd3", "#fda4af", "#fb7185", "#f43f5e", "#e11d48",
                "#be123c", "#9f1239", "#881337", "#4c0519",
            ],
        ),
    ];

    let shade_index = match shade {
        "50" => 0,
        "100" => 1,
        "200" => 2,
        "300" => 3,
        "400" => 4,
        "500" => 5,
        "600" => 6,
        "700" => 7,
        "800" => 8,
        "900" => 9,
        "950" => 10,
        _ => return None,
    };

    colors
        .iter()
        .find(|&&(c, _)| c == color)
        .and_then(|&(_, shades)| shades.get(shade_index).map(|&s| s.to_string()))
}

fn parse_display(class: &str) -> Option<String> {
    let display_values = [
        ("block", "block"),
        ("inline-block", "inline-block"),
        ("inline", "inline"),
        ("flex", "flex"),
        ("inline-flex", "inline-flex"),
        ("table", "table"),
        ("inline-table", "inline-table"),
        ("table-caption", "table-caption"),
        ("table-cell", "table-cell"),
        ("table-column", "table-column"),
        ("table-column-group", "table-column-group"),
        ("table-footer-group", "table-footer-group"),
        ("table-header-group", "table-header-group"),
        ("table-row-group", "table-row-group"),
        ("table-row", "table-row"),
        ("flow-root", "flow-root"),
        ("grid", "grid"),
        ("inline-grid", "inline-grid"),
        ("contents", "contents"),
        ("list-item", "list-item"),
        ("hidden", "none"),
    ];

    parse_property(class, "", "display", &display_values)
}

fn parse_flex(class: &str) -> Option<String> {
    let flex_values = [
        ("flex-row", "flex-direction: row;"),
        ("flex-row-reverse", "flex-direction: row-reverse;"),
        ("flex-col", "flex-direction: column;"),
        ("flex-col-reverse", "flex-direction: column-reverse;"),
        ("flex-wrap", "flex-wrap: wrap;"),
        ("flex-wrap-reverse", "flex-wrap: wrap-reverse;"),
        ("flex-nowrap", "flex-wrap: nowrap;"),
        ("flex-1", "flex: 1 1 0%;"),
        ("flex-auto", "flex: 1 1 auto;"),
        ("flex-initial", "flex: 0 1 auto;"),
        ("flex-none", "flex: none;"),
    ];

    flex_values
        .iter()
        .find(|&&(k, _)| k == class)
        .map(|&(_, v)| v.to_string())
        .or_else(|| {
            if class.starts_with("flex-grow-") {
                class
                    .strip_prefix("flex-grow-")
                    .map(|v| format!("flex-grow: {};", v))
            } else if class.starts_with("flex-shrink-") {
                class
                    .strip_prefix("flex-shrink-")
                    .map(|v| format!("flex-shrink: {};", v))
            } else {
                None
            }
        })
}

fn parse_grid(class: &str) -> Option<String> {
    if class.starts_with("grid-cols-") {
        return class
            .strip_prefix("grid-cols-")
            .map(|cols| format!("grid-template-columns: repeat({}, minmax(0, 1fr));", cols));
    }
    if class.starts_with("grid-rows-") {
        return class
            .strip_prefix("grid-rows-")
            .map(|rows| format!("grid-template-rows: repeat({}, minmax(0, 1fr));", rows));
    }

    let grid_values = [
        ("col-auto", "grid-column: auto;"),
        ("col-span-full", "grid-column: 1 / -1;"),
        ("col-start-auto", "grid-column-start: auto;"),
        ("col-end-auto", "grid-column-end: auto;"),
        ("row-auto", "grid-row: auto;"),
        ("row-span-full", "grid-row: 1 / -1;"),
        ("row-start-auto", "grid-row-start: auto;"),
        ("row-end-auto", "grid-row-end: auto;"),
    ];

    grid_values
        .iter()
        .find(|&&(k, _)| class.starts_with(k))
        .map(|&(_, v)| v.to_string())
        .or_else(|| {
            if class.starts_with("col-span-") {
                class
                    .strip_prefix("col-span-")
                    .map(|span| format!("grid-column: span {} / span {};", span, span))
            } else if class.starts_with("row-span-") {
                class
                    .strip_prefix("row-span-")
                    .map(|span| format!("grid-row: span {} / span {};", span, span))
            } else {
                None
            }
        })
}

fn parse_alignment(class: &str) -> Option<String> {
    let alignment_values = [
        ("justify-start", "justify-content: flex-start;"),
        ("justify-end", "justify-content: flex-end;"),
        ("justify-center", "justify-content: center;"),
        ("justify-between", "justify-content: space-between;"),
        ("justify-around", "justify-content: space-around;"),
        ("justify-evenly", "justify-content: space-evenly;"),
        ("justify-items-start", "justify-items: start;"),
        ("justify-items-end", "justify-items: end;"),
        ("justify-items-center", "justify-items: center;"),
        ("justify-items-stretch", "justify-items: stretch;"),
        ("justify-self-auto", "justify-self: auto;"),
        ("justify-self-start", "justify-self: start;"),
        ("justify-self-end", "justify-self: end;"),
        ("justify-self-center", "justify-self: center;"),
        ("justify-self-stretch", "justify-self: stretch;"),
        ("items-start", "align-items: flex-start;"),
        ("items-end", "align-items: flex-end;"),
        ("items-center", "align-items: center;"),
        ("items-baseline", "align-items: baseline;"),
        ("items-stretch", "align-items: stretch;"),
        ("content-center", "align-content: center;"),
        ("content-start", "align-content: flex-start;"),
        ("content-end", "align-content: flex-end;"),
        ("content-between", "align-content: space-between;"),
        ("content-around", "align-content: space-around;"),
        ("content-evenly", "align-content: space-evenly;"),
    ];

    alignment_values
        .iter()
        .find(|&&(k, _)| k == class)
        .map(|&(_, v)| v.to_string())
}

fn parse_layout(class: &str) -> Option<String> {
    let layout_values = [
        ("object-contain", "object-fit: contain;"),
        ("object-cover", "object-fit: cover;"),
        ("object-fill", "object-fit: fill;"),
        ("object-none", "object-fit: none;"),
        ("object-scale-down", "object-fit: scale-down;"),
        ("object-bottom", "object-position: bottom;"),
        ("object-center", "object-position: center;"),
        ("object-left", "object-position: left;"),
        ("object-left-bottom", "object-position: left bottom;"),
        ("object-left-top", "object-position: left top;"),
        ("object-right", "object-position: right;"),
        ("object-right-bottom", "object-position: right bottom;"),
        ("object-right-top", "object-position: right top;"),
        ("object-top", "object-position: top;"),
        ("overflow-auto", "overflow: auto;"),
        ("overflow-hidden", "overflow: hidden;"),
        ("overflow-clip", "overflow: clip;"),
        ("overflow-visible", "overflow: visible;"),
        ("overflow-scroll", "overflow: scroll;"),
        ("overflow-x-auto", "overflow-x: auto;"),
        ("overflow-y-auto", "overflow-y: auto;"),
        ("overflow-x-hidden", "overflow-x: hidden;"),
        ("overflow-y-hidden", "overflow-y: hidden;"),
        ("overflow-x-clip", "overflow-x: clip;"),
        ("overflow-y-clip", "overflow-y: clip;"),
        ("overflow-x-visible", "overflow-x: visible;"),
        ("overflow-y-visible", "overflow-y: visible;"),
        ("overflow-x-scroll", "overflow-x: scroll;"),
        ("overflow-y-scroll", "overflow-y: scroll;"),
        ("static", "position: static;"),
        ("fixed", "position: fixed;"),
        ("absolute", "position: absolute;"),
        ("relative", "position: relative;"),
        ("sticky", "position: sticky;"),
    ];

    for &(prefix, style) in &layout_values {
        if class == prefix {
            return Some(style.to_string());
        }
    }

    let position_properties = ["top", "right", "bottom", "left"];
    for prop in &position_properties {
        if class.starts_with(prop) {
            return parse_position_value(class, prop);
        }
    }

    None
}

fn parse_position_value(class: &str, property: &str) -> Option<String> {
    let value = class.strip_prefix(property)?;
    let value = value.strip_prefix('-')?;
    if value.is_empty() {
        return None;
    }

    let size_value = parse_size_value(value)?;

    Some(format!("{}: {};", property, size_value))
}

fn parse_size_value(value: &str) -> Option<String> {
    match value {
        "px" => Some("1px".to_string()),
        "0" => Some("0px".to_string()),
        "0.5" => Some("0.125rem".to_string()),
        "1" => Some("0.25rem".to_string()),
        "1.5" => Some("0.375rem".to_string()),
        "2" => Some("0.5rem".to_string()),
        "2.5" => Some("0.625rem".to_string()),
        "3" => Some("0.75rem".to_string()),
        "3.5" => Some("0.875rem".to_string()),
        "4" => Some("1rem".to_string()),
        "5" => Some("1.25rem".to_string()),
        "6" => Some("1.5rem".to_string()),
        "7" => Some("1.75rem".to_string()),
        "8" => Some("2rem".to_string()),
        "9" => Some("2.25rem".to_string()),
        "10" => Some("2.5rem".to_string()),
        "11" => Some("2.75rem".to_string()),
        "12" => Some("3rem".to_string()),
        "14" => Some("3.5rem".to_string()),
        "16" => Some("4rem".to_string()),
        "20" => Some("5rem".to_string()),
        "24" => Some("6rem".to_string()),
        "28" => Some("7rem".to_string()),
        "32" => Some("8rem".to_string()),
        "36" => Some("9rem".to_string()),
        "40" => Some("10rem".to_string()),
        "44" => Some("11rem".to_string()),
        "48" => Some("12rem".to_string()),
        "52" => Some("13rem".to_string()),
        "56" => Some("14rem".to_string()),
        "60" => Some("15rem".to_string()),
        "64" => Some("16rem".to_string()),
        "72" => Some("18rem".to_string()),
        "80" => Some("20rem".to_string()),
        "96" => Some("24rem".to_string()),
        "auto" => Some("auto".to_string()),
        "1/2" => Some("50%".to_string()),
        "1/3" => Some("33.333333%".to_string()),
        "2/3" => Some("66.666667%".to_string()),
        "1/4" => Some("25%".to_string()),
        "2/4" => Some("50%".to_string()),
        "3/4" => Some("75%".to_string()),
        "full" => Some("100%".to_string()),
        _ => None,
    }
}

fn parse_font_size(class: &str) -> Option<String> {
    let font_size_values = [
        ("text-xs", "0.75rem"),
        ("text-sm", "0.875rem"),
        ("text-base", "1rem"),
        ("text-lg", "1.125rem"),
        ("text-xl", "1.25rem"),
        ("text-2xl", "1.5rem"),
        ("text-3xl", "1.875rem"),
        ("text-4xl", "2.25rem"),
        ("text-5xl", "3rem"),
        ("text-6xl", "3.75rem"),
        ("text-7xl", "4.5rem"),
        ("text-8xl", "6rem"),
        ("text-9xl", "8rem"),
    ];

    parse_property(class, "", "font-size", &font_size_values).map(|style| {
        let size = style
            .split(':')
            .nth(1)
            .unwrap()
            .trim()
            .trim_end_matches(';');
        format!("font-size: {}; line-height: {};", size, size)
    })
}

fn parse_font_weight(class: &str) -> Option<String> {
    let font_weight_values = [
        ("font-thin", "100"),
        ("font-extralight", "200"),
        ("font-light", "300"),
        ("font-normal", "400"),
        ("font-medium", "500"),
        ("font-semibold", "600"),
        ("font-bold", "700"),
        ("font-extrabold", "800"),
        ("font-black", "900"),
    ];

    parse_property(class, "", "font-weight", &font_weight_values)
}

fn parse_letter_spacing(class: &str) -> Option<String> {
    let letter_spacing_values = [
        ("tracking-tighter", "-0.05em"),
        ("tracking-tight", "-0.025em"),
        ("tracking-normal", "0em"),
        ("tracking-wide", "0.025em"),
        ("tracking-wider", "0.05em"),
        ("tracking-widest", "0.1em"),
    ];

    parse_property(class, "", "letter-spacing", &letter_spacing_values)
}

fn parse_line_height(class: &str) -> Option<String> {
    let line_height_values = [
        ("leading-3", ".75rem"),
        ("leading-4", "1rem"),
        ("leading-5", "1.25rem"),
        ("leading-6", "1.5rem"),
        ("leading-7", "1.75rem"),
        ("leading-8", "2rem"),
        ("leading-9", "2.25rem"),
        ("leading-10", "2.5rem"),
        ("leading-none", "1"),
        ("leading-tight", "1.25"),
        ("leading-snug", "1.375"),
        ("leading-normal", "1.5"),
        ("leading-relaxed", "1.625"),
        ("leading-loose", "2"),
    ];

    parse_property(class, "", "line-height", &line_height_values)
}

fn parse_list_style_type(class: &str) -> Option<String> {
    let list_style_type_values = [
        ("list-none", "none"),
        ("list-disc", "disc"),
        ("list-decimal", "decimal"),
    ];

    parse_property(class, "", "list-style-type", &list_style_type_values)
}

fn parse_list_style_position(class: &str) -> Option<String> {
    let list_style_position_values = [("list-inside", "inside"), ("list-outside", "outside")];

    parse_property(
        class,
        "",
        "list-style-position",
        &list_style_position_values,
    )
}

fn parse_placeholder_color(class: &str) -> Option<String> {
    if class.starts_with("placeholder-") {
        parse_color(class).map(|color| color.replace("color", "color"))
    } else {
        None
    }
}

fn parse_placeholder_opacity(class: &str) -> Option<String> {
    parse_property(
        class,
        "placeholder-opacity-",
        "opacity",
        &[
            ("0", "0"),
            ("5", "0.05"),
            ("10", "0.1"),
            ("20", "0.2"),
            ("25", "0.25"),
            ("30", "0.3"),
            ("40", "0.4"),
            ("50", "0.5"),
            ("60", "0.6"),
            ("70", "0.7"),
            ("75", "0.75"),
            ("80", "0.8"),
            ("90", "0.9"),
            ("95", "0.95"),
            ("100", "1"),
        ],
    )
    .map(|style| format!("::placeholder {{ {} }}", style))
}

fn parse_text_align(class: &str) -> Option<String> {
    let text_align_values = [
        ("text-left", "left"),
        ("text-center", "center"),
        ("text-right", "right"),
        ("text-justify", "justify"),
    ];

    parse_property(class, "", "text-align", &text_align_values)
}

fn parse_text_decoration(class: &str) -> Option<String> {
    let text_decoration_values = [
        ("underline", "underline"),
        ("overline", "overline"),
        ("line-through", "line-through"),
        ("no-underline", "none"),
    ];

    parse_property(class, "", "text-decoration", &text_decoration_values)
}

fn parse_text_transform(class: &str) -> Option<String> {
    let text_transform_values = [
        ("uppercase", "uppercase"),
        ("lowercase", "lowercase"),
        ("capitalize", "capitalize"),
        ("normal-case", "none"),
    ];

    parse_property(class, "", "text-transform", &text_transform_values)
}

fn parse_vertical_align(class: &str) -> Option<String> {
    let vertical_align_values = [
        ("align-baseline", "baseline"),
        ("align-top", "top"),
        ("align-middle", "middle"),
        ("align-bottom", "bottom"),
        ("align-text-top", "text-top"),
        ("align-text-bottom", "text-bottom"),
    ];

    parse_property(class, "", "vertical-align", &vertical_align_values)
}

fn parse_whitespace(class: &str) -> Option<String> {
    let whitespace_values = [
        ("whitespace-normal", "normal"),
        ("whitespace-nowrap", "nowrap"),
        ("whitespace-pre", "pre"),
        ("whitespace-pre-line", "pre-line"),
        ("whitespace-pre-wrap", "pre-wrap"),
    ];

    parse_property(class, "", "white-space", &whitespace_values)
}

fn parse_word_break(class: &str) -> Option<String> {
    match class {
        "break-normal" => Some("overflow-wrap: normal; word-break: normal;".to_string()),
        "break-words" => Some("overflow-wrap: break-word;".to_string()),
        "break-all" => Some("word-break: break-all;".to_string()),
        _ => None,
    }
}

fn parse_font(class: &str) -> Option<String> {
    let font_values = [
        ("font-sans", "font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, \"Helvetica Neue\", Arial, \"Noto Sans\", sans-serif, \"Apple Color Emoji\", \"Segoe UI Emoji\", \"Segoe UI Symbol\", \"Noto Color Emoji\";"),
        ("font-serif", "font-family: ui-serif, Georgia, Cambria, \"Times New Roman\", Times, serif;"),
        ("font-mono", "font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, \"Liberation Mono\", \"Courier New\", monospace;"),
        ("font-thin", "font-weight: 100;"),
        ("font-extralight", "font-weight: 200;"),
        ("font-light", "font-weight: 300;"),
        ("font-normal", "font-weight: 400;"),
        ("font-medium", "font-weight: 500;"),
        ("font-semibold", "font-weight: 600;"),
        ("font-bold", "font-weight: 700;"),
        ("font-extrabold", "font-weight: 800;"),
        ("font-black", "font-weight: 900;"),
    ];

    font_values
        .iter()
        .find(|&&(k, _)| k == class)
        .map(|&(_, v)| v.to_string())
}

fn parse_border(class: &str) -> Option<String> {
    let border_values = [
        ("border", "border-width: 1px;"),
        ("border-0", "border-width: 0px;"),
        ("border-2", "border-width: 2px;"),
        ("border-4", "border-width: 4px;"),
        ("border-8", "border-width: 8px;"),
        ("border-t", "border-top-width: 1px;"),
        ("border-r", "border-right-width: 1px;"),
        ("border-b", "border-bottom-width: 1px;"),
        ("border-l", "border-left-width: 1px;"),
    ];

    border_values
        .iter()
        .find(|&&(k, _)| k == class)
        .map(|&(_, v)| v.to_string())
        .or_else(|| {
            let parts: Vec<&str> = class.split('-').collect();
            match (parts.get(0), parts.get(1), parts.get(2)) {
                (Some(&"border"), Some(&"t" | &"r" | &"b" | &"l"), Some(&"0")) => Some(format!(
                    "border-{}-width: 0px;",
                    match parts[1] {
                        "t" => "top",
                        "r" => "right",
                        "b" => "bottom",
                        "l" => "left",
                        _ => unreachable!(),
                    }
                )),
                _ => None,
            }
        })
        .or_else(|| parse_color(class))
}

fn parse_border_radius(class: &str) -> Option<String> {
    let radius_values = [
        ("rounded-none", "border-radius: 0px;"),
        ("rounded-sm", "border-radius: 0.125rem;"),
        ("rounded", "border-radius: 0.25rem;"),
        ("rounded-md", "border-radius: 0.375rem;"),
        ("rounded-lg", "border-radius: 0.5rem;"),
        ("rounded-xl", "border-radius: 0.75rem;"),
        ("rounded-2xl", "border-radius: 1rem;"),
        ("rounded-3xl", "border-radius: 1.5rem;"),
        ("rounded-full", "border-radius: 9999px;"),
    ];

    radius_values
        .iter()
        .find(|&&(k, _)| k == class)
        .map(|&(_, v)| v.to_string())
        .or_else(|| {
            let parts: Vec<&str> = class.split('-').collect();
            match (parts.get(0), parts.get(1), parts.get(2)) {
                (Some(&"rounded"), Some(&"t" | &"r" | &"b" | &"l"), Some(size)) => {
                    let (first, second) = match parts[1] {
                        "t" => ("top-left", "top-right"),
                        "r" => ("top-right", "bottom-right"),
                        "b" => ("bottom-left", "bottom-right"),
                        "l" => ("top-left", "bottom-left"),
                        _ => unreachable!(),
                    };
                    let value = parse_value(size, &radius_values)?;
                    Some(format!(
                        "border-{}-radius: {}; border-{}-radius: {};",
                        first, value, second, value
                    ))
                }
                _ => None,
            }
        })
}

fn parse_shadow(class: &str) -> Option<String> {
    let shadow_values = [
        ("shadow-sm", "box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05);"),
        (
            "shadow",
            "box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1);",
        ),
        (
            "shadow-md",
            "box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);",
        ),
        (
            "shadow-lg",
            "box-shadow: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1);",
        ),
        (
            "shadow-xl",
            "box-shadow: 0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1);",
        ),
        (
            "shadow-2xl",
            "box-shadow: 0 25px 50px -12px rgb(0 0 0 / 0.25);",
        ),
        (
            "shadow-inner",
            "box-shadow: inset 0 2px 4px 0 rgb(0 0 0 / 0.05);",
        ),
        ("shadow-none", "box-shadow: 0 0 #0000;"),
    ];

    shadow_values
        .iter()
        .find(|&&(k, _)| k == class)
        .map(|&(_, v)| v.to_string())
}

fn parse_opacity(class: &str) -> Option<String> {
    parse_property(
        class,
        "opacity-",
        "opacity",
        &[
            ("0", "0"),
            ("5", "0.05"),
            ("10", "0.1"),
            ("20", "0.2"),
            ("25", "0.25"),
            ("30", "0.3"),
            ("40", "0.4"),
            ("50", "0.5"),
            ("60", "0.6"),
            ("70", "0.7"),
            ("75", "0.75"),
            ("80", "0.8"),
            ("90", "0.9"),
            ("95", "0.95"),
            ("100", "1"),
        ],
    )
}

fn parse_z_index(class: &str) -> Option<String> {
    parse_property(
        class,
        "z-",
        "z-index",
        &[
            ("0", "0"),
            ("10", "10"),
            ("20", "20"),
            ("30", "30"),
            ("40", "40"),
            ("50", "50"),
            ("auto", "auto"),
        ],
    )
}

fn parse_animation(class: &str) -> Option<String> {
    let animation_values = [
        ("animate-none", "animation: none;"),
        ("animate-spin", "animation: spin 1s linear infinite;"),
        (
            "animate-ping",
            "animation: ping 1s cubic-bezier(0, 0, 0.2, 1) infinite;",
        ),
        (
            "animate-pulse",
            "animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;",
        ),
        ("animate-bounce", "animation: bounce 1s infinite;"),
    ];

    animation_values
        .iter()
        .find(|&&(k, _)| k == class)
        .map(|&(_, v)| v.to_string())
}

fn parse_text(class: &str) -> Option<String> {
    if class.starts_with("text-") {
        let value = &class[5..]; // Remove "text-" prefix

        // First, try to parse it as a color
        if let Some(color_style) = parse_color(class) {
            return Some(color_style);
        }

        // If it's not a color, check if it's a size
        let text_sizes = [
            ("xs", "0.75rem"),
            ("sm", "0.875rem"),
            ("base", "1rem"),
            ("lg", "1.125rem"),
            ("xl", "1.25rem"),
            ("2xl", "1.5rem"),
            ("3xl", "1.875rem"),
            ("4xl", "2.25rem"),
            ("5xl", "3rem"),
            ("6xl", "3.75rem"),
            ("7xl", "4.5rem"),
            ("8xl", "6rem"),
            ("9xl", "8rem"),
        ];

        for &(size, size_value) in &text_sizes {
            if value == size {
                return Some(format!("font-size: {};", size_value));
            }
        }

        // If it's neither a color nor a predefined size, it might be an arbitrary value
        if value.starts_with('[') && value.ends_with(']') {
            let v = &value[1..value.len() - 1];
            if size_like(v) {
                return Some(format!("font-size: {v};"));
            } else {
                return Some(format!("color: {v};"));
            }
        }
    }

    // Handle other text-related classes
    match class {
        "text-left" => Some("text-align: left;".to_string()),
        "text-center" => Some("text-align: center;".to_string()),
        "text-right" => Some("text-align: right;".to_string()),
        "text-justify" => Some("text-align: justify;".to_string()),
        "underline" => Some("text-decoration: underline;".to_string()),
        "overline" => Some("text-decoration: overline;".to_string()),
        "line-through" => Some("text-decoration: line-through;".to_string()),
        "no-underline" => Some("text-decoration: none;".to_string()),
        "uppercase" => Some("text-transform: uppercase;".to_string()),
        "lowercase" => Some("text-transform: lowercase;".to_string()),
        "capitalize" => Some("text-transform: capitalize;".to_string()),
        "normal-case" => Some("text-transform: none;".to_string()),
        "truncate" => {
            Some("overflow: hidden; text-overflow: ellipsis; white-space: nowrap;".to_string())
        }
        "antialiased" => Some(
            "-webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale;".to_string(),
        ),
        "subpixel-antialiased" => {
            Some("-webkit-font-smoothing: auto; -moz-osx-font-smoothing: auto;".to_string())
        }
        _ => None,
    }
}

fn parse_typography(class: &str) -> Option<String> {
    // Handle prose classes
    if class.starts_with("prose") {
        let prose_styles = vec!["font-size: 1rem;", "line-height: 1.75;"];

        let mut styles = prose_styles.join(" ");

        // Handle prose size modifiers
        match class {
            "prose-sm" => styles += " font-size: 0.875rem;",
            "prose-base" => styles += " font-size: 1rem;",
            "prose-lg" => styles += " font-size: 1.125rem;",
            "prose-xl" => styles += " font-size: 1.25rem;",
            "prose-2xl" => styles += " font-size: 1.5rem;",
            _ => {}
        }

        return Some(styles);
    }

    // Handle other typography classes
    match class {
        "font-sans" => Some("font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, \"Helvetica Neue\", Arial, \"Noto Sans\", sans-serif, \"Apple Color Emoji\", \"Segoe UI Emoji\", \"Segoe UI Symbol\", \"Noto Color Emoji\";".to_string()),
        "font-serif" => Some("font-family: ui-serif, Georgia, Cambria, \"Times New Roman\", Times, serif;".to_string()),
        "font-mono" => Some("font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, \"Liberation Mono\", \"Courier New\", monospace;".to_string()),
        "italic" => Some("font-style: italic;".to_string()),
        "not-italic" => Some("font-style: normal;".to_string()),
        "ordinal" => Some("font-variant-numeric: ordinal;".to_string()),
        "slashed-zero" => Some("font-variant-numeric: slashed-zero;".to_string()),
        "lining-nums" => Some("font-variant-numeric: lining-nums;".to_string()),
        "oldstyle-nums" => Some("font-variant-numeric: oldstyle-nums;".to_string()),
        "proportional-nums" => Some("font-variant-numeric: proportional-nums;".to_string()),
        "tabular-nums" => Some("font-variant-numeric: tabular-nums;".to_string()),
        "diagonal-fractions" => Some("font-variant-numeric: diagonal-fractions;".to_string()),
        "stacked-fractions" => Some("font-variant-numeric: stacked-fractions;".to_string()),
        _ => None,
    }
}

fn parse_background(class: &str) -> Option<String> {
    let bg_values = [
        ("bg-fixed", "background-attachment: fixed;"),
        ("bg-local", "background-attachment: local;"),
        ("bg-scroll", "background-attachment: scroll;"),
        ("bg-clip-border", "background-clip: border-box;"),
        ("bg-clip-padding", "background-clip: padding-box;"),
        ("bg-clip-content", "background-clip: content-box;"),
        ("bg-clip-text", "background-clip: text;"),
        ("bg-repeat", "background-repeat: repeat;"),
        ("bg-no-repeat", "background-repeat: no-repeat;"),
        ("bg-repeat-x", "background-repeat: repeat-x;"),
        ("bg-repeat-y", "background-repeat: repeat-y;"),
        ("bg-repeat-round", "background-repeat: round;"),
        ("bg-repeat-space", "background-repeat: space;"),
        ("bg-origin-border", "background-origin: border-box;"),
        ("bg-origin-padding", "background-origin: padding-box;"),
        ("bg-origin-content", "background-origin: content-box;"),
        ("bg-none", "background-image: none;"),
    ];

    for &(prefix, style) in &bg_values {
        if class == prefix {
            return Some(style.to_string());
        }
    }

    if class.starts_with("bg-gradient-to-") {
        let direction = class.strip_prefix("bg-gradient-to-").unwrap();
        let gradient_direction = match direction {
            "t" => "to top",
            "tr" => "to top right",
            "r" => "to right",
            "br" => "to bottom right",
            "b" => "to bottom",
            "bl" => "to bottom left",
            "l" => "to left",
            "tl" => "to top left",
            _ => return None,
        };
        return Some(format!(
            "background-image: linear-gradient({}, transparent, transparent);",
            gradient_direction
        ));
    }

    if class.starts_with("bg-") {
        return parse_color(class);
    }

    None
}

fn parse_gradient_color_stop(class: &str) -> Option<GradientStop> {
    let parts: Vec<&str> = class.split('-').collect();
    if parts.len() < 4 || parts[0] != "from" && parts[0] != "via" && parts[0] != "to" {
        return None;
    }

    let position = match parts[0] {
        "from" => "0%",
        "via" => "50%",
        "to" => "100%",
        _ => return None,
    };

    let color = parse_color(&format!("bg-{}-{}", parts[1], parts[2]))?;
    let color_value = color.split(':').nth(1)?.trim().trim_end_matches(';');

    Some(GradientStop {
        color: color_value.to_string(),
        position: position.to_string(),
    })
}

fn parse_transition(class: &str) -> Option<String> {
    let transition_values = [
        ("transition-none", "transition-property: none;"),
        ("transition-all", "transition-property: all; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;"),
        ("transition", "transition-property: color, background-color, border-color, text-decoration-color, fill, stroke, opacity, box-shadow, transform, filter, backdrop-filter; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;"),
        ("transition-colors", "transition-property: color, background-color, border-color, text-decoration-color, fill, stroke; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;"),
        ("transition-opacity", "transition-property: opacity; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;"),
        ("transition-shadow", "transition-property: box-shadow; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;"),
        ("transition-transform", "transition-property: transform; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;"),
    ];

    transition_values
        .iter()
        .find(|&&(k, _)| k == class)
        .map(|&(_, v)| v.to_string())
}

fn parse_transform(class: &str) -> Option<String> {
    if class.starts_with("scale-") {
        let scale_value = class.strip_prefix("scale-")?;

        // Check if it's a single numeric value
        if let Ok(scale) = scale_value.parse::<f32>() {
            return Some(format!("transform: scale({});", scale / 100.0));
        }

        // Check if it's an arbitrary value (possibly with multiple values)
        if scale_value.starts_with('[') && scale_value.ends_with(']') {
            let inner = &scale_value[1..scale_value.len() - 1];
            let values: Vec<&str> = inner.split(',').collect();

            if values.len() == 1 {
                // Single arbitrary value
                if let Ok(scale) = inner.parse::<f32>() {
                    return Some(format!("transform: scale({});", scale));
                }
            } else if values.len() == 2 {
                // Two values for x and y scaling
                if let (Ok(scale_x), Ok(scale_y)) =
                    (values[0].parse::<f32>(), values[1].parse::<f32>())
                {
                    return Some(format!("transform: scale({}, {});", scale_x, scale_y));
                }
            }
        }
    }
    if class.starts_with("rotate-") {
        return class
            .strip_prefix("rotate-")
            .and_then(|deg| deg.parse::<i32>().ok())
            .map(|deg| format!("transform: rotate({}deg);", deg));
    }
    if class.starts_with("translate-") {
        let parts: Vec<&str> = class.split('-').collect();
        if parts.len() == 3 {
            let direction = parts[1];
            let amount = parts[2];
            let value = parse_value(
                amount,
                &[
                    ("px", "1px"),
                    ("0", "0px"),
                    ("0.5", "0.125rem"),
                    ("1", "0.25rem"),
                    ("1.5", "0.375rem"),
                    ("2", "0.5rem"),
                    ("2.5", "0.625rem"),
                    ("3", "0.75rem"),
                    ("3.5", "0.875rem"),
                    ("4", "1rem"),
                ],
            )?;
            return Some(format!(
                "transform: translate{}({});",
                direction.to_uppercase(),
                value
            ));
        }
    }
    None
}

fn parse_gap(class: &str) -> Option<String> {
    let gap_values = [
        ("gap-0", "0px"),
        ("gap-x-0", "0px"),
        ("gap-y-0", "0px"),
        ("gap-px", "1px"),
        ("gap-x-px", "1px"),
        ("gap-y-px", "1px"),
        ("gap-0.5", "0.125rem"),
        ("gap-x-0.5", "0.125rem"),
        ("gap-y-0.5", "0.125rem"),
        ("gap-1", "0.25rem"),
        ("gap-x-1", "0.25rem"),
        ("gap-y-1", "0.25rem"),
        ("gap-1.5", "0.375rem"),
        ("gap-x-1.5", "0.375rem"),
        ("gap-y-1.5", "0.375rem"),
        ("gap-2", "0.5rem"),
        ("gap-x-2", "0.5rem"),
        ("gap-y-2", "0.5rem"),
        ("gap-2.5", "0.625rem"),
        ("gap-x-2.5", "0.625rem"),
        ("gap-y-2.5", "0.625rem"),
        ("gap-3", "0.75rem"),
        ("gap-x-3", "0.75rem"),
        ("gap-y-3", "0.75rem"),
        ("gap-3.5", "0.875rem"),
        ("gap-x-3.5", "0.875rem"),
        ("gap-y-3.5", "0.875rem"),
        ("gap-4", "1rem"),
        ("gap-x-4", "1rem"),
        ("gap-y-4", "1rem"),
        ("gap-5", "1.25rem"),
        ("gap-x-5", "1.25rem"),
        ("gap-y-5", "1.25rem"),
        ("gap-6", "1.5rem"),
        ("gap-x-6", "1.5rem"),
        ("gap-y-6", "1.5rem"),
        ("gap-7", "1.75rem"),
        ("gap-x-7", "1.75rem"),
        ("gap-y-7", "1.75rem"),
        ("gap-8", "2rem"),
        ("gap-x-8", "2rem"),
        ("gap-y-8", "2rem"),
        ("gap-9", "2.25rem"),
        ("gap-x-9", "2.25rem"),
        ("gap-y-9", "2.25rem"),
        ("gap-10", "2.5rem"),
        ("gap-x-10", "2.5rem"),
        ("gap-y-10", "2.5rem"),
        ("gap-11", "2.75rem"),
        ("gap-x-11", "2.75rem"),
        ("gap-y-11", "2.75rem"),
        ("gap-12", "3rem"),
        ("gap-x-12", "3rem"),
        ("gap-y-12", "3rem"),
        ("gap-14", "3.5rem"),
        ("gap-x-14", "3.5rem"),
        ("gap-y-14", "3.5rem"),
        ("gap-16", "4rem"),
        ("gap-x-16", "4rem"),
        ("gap-y-16", "4rem"),
        ("gap-20", "5rem"),
        ("gap-x-20", "5rem"),
        ("gap-y-20", "5rem"),
        ("gap-24", "6rem"),
        ("gap-x-24", "6rem"),
        ("gap-y-24", "6rem"),
        ("gap-28", "7rem"),
        ("gap-x-28", "7rem"),
        ("gap-y-28", "7rem"),
        ("gap-32", "8rem"),
        ("gap-x-32", "8rem"),
        ("gap-y-32", "8rem"),
        ("gap-36", "9rem"),
        ("gap-x-36", "9rem"),
        ("gap-y-36", "9rem"),
        ("gap-40", "10rem"),
        ("gap-x-40", "10rem"),
        ("gap-y-40", "10rem"),
        ("gap-44", "11rem"),
        ("gap-x-44", "11rem"),
        ("gap-y-44", "11rem"),
        ("gap-48", "12rem"),
        ("gap-x-48", "12rem"),
        ("gap-y-48", "12rem"),
        ("gap-52", "13rem"),
        ("gap-x-52", "13rem"),
        ("gap-y-52", "13rem"),
        ("gap-56", "14rem"),
        ("gap-x-56", "14rem"),
        ("gap-y-56", "14rem"),
        ("gap-60", "15rem"),
        ("gap-x-60", "15rem"),
        ("gap-y-60", "15rem"),
        ("gap-64", "16rem"),
        ("gap-x-64", "16rem"),
        ("gap-y-64", "16rem"),
        ("gap-72", "18rem"),
        ("gap-x-72", "18rem"),
        ("gap-y-72", "18rem"),
        ("gap-80", "20rem"),
        ("gap-x-80", "20rem"),
        ("gap-y-80", "20rem"),
        ("gap-96", "24rem"),
        ("gap-x-96", "24rem"),
        ("gap-y-96", "24rem"),
    ];

    // Check for predefined gap values
    if let Some(style) = parse_property(class, "", "gap", &gap_values) {
        return Some(style);
    }

    // Handle arbitrary values
    if class.starts_with("gap-[") || class.starts_with("gap-x-[") || class.starts_with("gap-y-[") {
        let (prefix, property) = if class.starts_with("gap-x-") {
            ("gap-x-", "column-gap")
        } else if class.starts_with("gap-y-") {
            ("gap-y-", "row-gap")
        } else {
            ("gap-", "gap")
        };

        if let Some(value) = class.strip_prefix(prefix) {
            if value.starts_with('[') && value.ends_with(']') {
                let arbitrary_value = &value[1..value.len() - 1];
                return Some(format!("{}: {};", property, arbitrary_value));
            }
        }
    }

    None
}

fn parse_filter(class: &str) -> Option<String> {
    let filter_values = [
        ("blur", "blur"),
        ("brightness", "brightness"),
        ("contrast", "contrast"),
        ("grayscale", "grayscale"),
        ("hue-rotate", "hue-rotate"),
        ("invert", "invert"),
        ("saturate", "saturate"),
        ("sepia", "sepia"),
    ];

    for (prefix, filter_type) in filter_values.iter() {
        if class.starts_with(prefix) {
            let value = class.strip_prefix(prefix).unwrap();
            let filter_value = match *filter_type {
                "blur" => parse_blur_value(value)?,
                "hue-rotate" => parse_angle_value(value)?,
                _ => parse_percentage_value(value)?,
            };
            return Some(format!("filter: {}({});", filter_type, filter_value));
        }
    }
    None
}

fn parse_backdrop_filter(class: &str) -> Option<String> {
    let backdrop_filter_values = [
        ("backdrop-blur", "blur"),
        ("backdrop-brightness", "brightness"),
        ("backdrop-contrast", "contrast"),
        ("backdrop-grayscale", "grayscale"),
        ("backdrop-hue-rotate", "hue-rotate"),
        ("backdrop-invert", "invert"),
        ("backdrop-opacity", "opacity"),
        ("backdrop-saturate", "saturate"),
        ("backdrop-sepia", "sepia"),
    ];

    for (prefix, filter_type) in backdrop_filter_values.iter() {
        if class.starts_with(prefix) {
            let value = class.strip_prefix(prefix).unwrap();
            let filter_value = match *filter_type {
                "blur" => parse_blur_value(value)?,
                "hue-rotate" => parse_angle_value(value)?,
                _ => parse_percentage_value(value)?,
            };
            return Some(format!(
                "backdrop-filter: {}({});",
                filter_type, filter_value
            ));
        }
    }
    None
}

fn parse_blur_value(value: &str) -> Option<String> {
    match value {
        "none" => Some("0".to_string()),
        "sm" => Some("4px".to_string()),
        "md" => Some("12px".to_string()),
        "lg" => Some("16px".to_string()),
        "xl" => Some("24px".to_string()),
        "2xl" => Some("40px".to_string()),
        "3xl" => Some("64px".to_string()),
        _ => None,
    }
}

fn parse_percentage_value(value: &str) -> Option<String> {
    value.parse::<u32>().ok().map(|v| format!("{}%", v))
}

fn parse_angle_value(value: &str) -> Option<String> {
    value.parse::<i32>().ok().map(|v| format!("{}deg", v))
}

fn parse_arbitrary_value(class: &str) -> Option<String> {
    let re = Regex::new(r"^([a-zA-Z-]+)\[(.*?)\]$").unwrap();
    if let Some(captures) = re.captures(class) {
        let property = captures.get(1)?.as_str();
        let value = captures.get(2)?.as_str();
        match property {
            "w" => Some(format!("width: {};", value)),
            "h" => Some(format!("height: {};", value)),
            "text" => Some(format!("font-size: {};", value)),
            "bg" => Some(format!("background-color: {};", value)),
            "p" => Some(format!("padding: {};", value)),
            "m" => Some(format!("margin: {};", value)),
            "gap" => Some(format!("gap: {};", value)),
            "top" | "right" | "bottom" | "left" => Some(format!("{}: {};", property, value)),
            "translate-x" => Some(format!("transform: translateX({});", value)),
            "translate-y" => Some(format!("transform: translateY({});", value)),
            "rotate" => Some(format!("transform: rotate({});", value)),
            "scale" => Some(format!("transform: scale({});", value)),
            "skew-x" => Some(format!("transform: skewX({});", value)),
            "skew-y" => Some(format!("transform: skewY({});", value)),
            _ => None,
        }
    } else {
        None
    }
}

fn nest_styles(styles: HashMap<String, Vec<String>>, element_id: &str) -> String {
    let mut nested: HashMap<String, NestedStyles> = HashMap::new();

    let mut styles: Vec<(String, Vec<String>)> = styles.into_iter().collect();
    styles.sort_by(|(v1, _), (v2, _)| {
        fn to_order(v: &str) -> u8 {
            match v {
                v if v.contains("sm") => 1,
                v if v.contains("md") => 2,
                v if v.contains("lg") => 3,
                v if v.contains("xl") => 4,
                v if v.contains("2xl") => 5,
                _ => 0,
            }
        }
        to_order(v1).cmp(&to_order(v2))
    });

    for (variants, styles) in styles {
        let parts: Vec<&str> = variants.split(':').collect();
        insert_nested(&mut nested, &parts, styles);
    }

    format_nested_styles(&nested, element_id, "")
}

fn insert_nested(nested: &mut HashMap<String, NestedStyles>, parts: &[&str], styles: Vec<String>) {
    if parts.is_empty() {
        return;
    }

    let selector = variant_to_selector(parts[0]);

    let entry = nested
        .entry(selector)
        .or_insert_with(|| NestedStyles::Nested(HashMap::new()));

    match entry {
        NestedStyles::Styles(existing_styles) => {
            existing_styles.extend(styles);
        }
        NestedStyles::Nested(next_level) => {
            if parts.len() == 1 {
                next_level
                    .entry("".to_string())
                    .or_insert_with(|| NestedStyles::Styles(Vec::new()))
                    .insert_styles(&[], styles);
            } else {
                insert_nested(next_level, &parts[1..], styles);
            }
        }
    }
}

fn variant_to_selector(variant: &str) -> String {
    match variant {
        "sm" => "@media (min-width: 640px)".to_string(),
        "md" => "@media (min-width: 768px)".to_string(),
        "lg" => "@media (min-width: 1024px)".to_string(),
        "xl" => "@media (min-width: 1280px)".to_string(),
        "2xl" => "@media (min-width: 1536px)".to_string(),
        "hover" => ":hover".to_string(),
        "focus" => ":focus".to_string(),
        "active" => ":active".to_string(),
        "disabled" => ":disabled".to_string(),
        "first" => ":first-child".to_string(),
        "last" => ":last-child".to_string(),
        "odd" => ":nth-child(odd)".to_string(),
        "even" => ":nth-child(even)".to_string(),
        "dark" => "@media (prefers-color-scheme: dark)".to_string(),
        _ => variant.to_string(),
    }
}

impl NestedStyles {
    fn insert_styles(&mut self, parts: &[&str], styles: Vec<String>) {
        match self {
            NestedStyles::Styles(existing_styles) => existing_styles.extend(styles),
            NestedStyles::Nested(next_level) => {
                if parts.is_empty() {
                    next_level
                        .entry("".to_string())
                        .or_insert_with(|| NestedStyles::Styles(Vec::new()))
                        .insert_styles(&[], styles);
                } else {
                    insert_nested(next_level, parts, styles);
                }
            }
        }
    }
}

fn format_nested_styles(
    styles: &HashMap<String, NestedStyles>,
    element_id: &str,
    parent_selector: &str,
) -> String {
    let mut result = String::new();

    for (selector, content) in styles {
        let current_selector = if parent_selector.is_empty() {
            if selector.is_empty() {
                format!("#{}", element_id)
            } else if selector.starts_with(':') {
                format!("#{}{}", element_id, selector)
            } else {
                format!("#{}", element_id)
            }
        } else if selector.starts_with('@') {
            parent_selector.to_string()
        } else if selector.is_empty() {
            parent_selector.to_string()
        } else {
            format!("{}{}", parent_selector, selector)
        };

        match content {
            NestedStyles::Styles(s) => {
                let resolved_styles = resolve_conflicts(s.clone());
                if selector.starts_with('@') {
                    result += &format!(
                        "{} {{\n  {} {{\n    {}\n  }}\n}}\n",
                        selector,
                        current_selector.trim(),
                        resolved_styles.join("\n    ")
                    );
                } else {
                    result += &format!(
                        "{} {{\n  {}\n}}\n",
                        current_selector.trim(),
                        resolved_styles.join("\n  ")
                    );
                }
            }
            NestedStyles::Nested(nested) => {
                if selector.starts_with('@') {
                    result += &format!(
                        "{} {{\n{}}}\n",
                        selector,
                        format_nested_styles(nested, element_id, &current_selector)
                    );
                } else {
                    result += &format_nested_styles(nested, element_id, &current_selector);
                }
            }
        }
    }

    result
}

fn resolve_conflicts(styles: Vec<String>) -> Vec<String> {
    let mut property_map: HashMap<String, Vec<String>> = HashMap::new();

    for style in styles {
        let parts: Vec<&str> = style.splitn(2, ':').collect();
        if parts.len() == 2 {
            let property = parts[0].trim();
            let value = parts[1].trim().trim_end_matches(';');
            property_map
                .entry(property.to_string())
                .or_insert_with(Vec::new)
                .push(value.to_string());
        }
    }

    property_map
        .into_iter()
        .map(|(property, values)| {
            if property == "transform" {
                format!("{}: {};", property, values.join(" "))
            } else {
                format!("{}: {};", property, values.last().unwrap())
            }
        })
        .collect()
}

fn size_like(s: &str) -> bool {
    s.starts_with(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'])
}

#[derive(Debug)]
enum NestedStyles {
    Styles(Vec<String>),
    Nested(HashMap<String, NestedStyles>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_tailwind() {
        let input = "max-w-screen-md bg-blue-500 hover:bg-red-500 sm:hover:bg-green-500 lg:p-4 sm:p-2 w-full text-white hover:text-black sm:w-1/2 md:w-1/3";
        let (inline_styles, complex_styles) = compile(input, "myelement");
        println!("Input: {}", input);
        println!("Inline styles: {}", inline_styles.unwrap());
        println!("Complex styles:\n{}", complex_styles.unwrap());
    }
}
