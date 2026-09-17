use std::{error::Error, fmt};

const SUPPORTED_PROPERTIES: [&str; 3] = ["bc", "ccc", "ea"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyValueAlias {
    property: String,
    canonical: String,
    abbreviation: String,
    long_name: String,
}

impl PropertyValueAlias {
    pub fn property(&self) -> &str {
        &self.property
    }

    pub fn canonical(&self) -> &str {
        &self.canonical
    }

    pub fn abbreviation(&self) -> &str {
        &self.abbreviation
    }

    pub fn long_name(&self) -> &str {
        &self.long_name
    }

    pub fn matches(&self, value: &str) -> bool {
        self.canonical.eq_ignore_ascii_case(value)
            || self.abbreviation.eq_ignore_ascii_case(value)
            || self.long_name.eq_ignore_ascii_case(value)
    }
}

pub fn parse(source: &str) -> Result<Vec<PropertyValueAlias>, ParseError> {
    let mut aliases = Vec::new();

    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        let content = line.split_once('#').map_or(line, |item| item.0).trim();
        if content.is_empty() {
            continue;
        }

        let fields = content.split(';').map(str::trim).collect::<Vec<_>>();
        let Some(property) = fields.first() else {
            continue;
        };
        if !SUPPORTED_PROPERTIES.contains(property) {
            continue;
        }

        let expected_count = if *property == "ccc" { 4 } else { 3 };
        if fields.len() != expected_count {
            return Err(ParseError::new(
                line_number,
                format!(
                    "property `{property}` expects {expected_count} fields, found {}",
                    fields.len()
                ),
            ));
        }
        if let Some((field_index, _)) = fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.is_empty())
        {
            return Err(ParseError::new(
                line_number,
                format!("field {} is empty", field_index + 1),
            ));
        }

        let (canonical, abbreviation, long_name) = if *property == "ccc" {
            (fields[1], fields[2], fields[3])
        } else {
            (fields[1], fields[1], fields[2])
        };
        let alias = PropertyValueAlias {
            property: (*property).to_owned(),
            canonical: canonical.to_owned(),
            abbreviation: abbreviation.to_owned(),
            long_name: long_name.to_owned(),
        };
        if aliases.iter().any(|existing: &PropertyValueAlias| {
            existing.property == alias.property
                && (existing.matches(alias.canonical())
                    || existing.matches(alias.abbreviation())
                    || existing.matches(alias.long_name()))
        }) {
            return Err(ParseError::new(
                line_number,
                format!("property `{property}` has an ambiguous alias `{canonical}`"),
            ));
        }
        aliases.push(alias);
    }

    for property in SUPPORTED_PROPERTIES {
        if !aliases.iter().any(|alias| alias.property == property) {
            return Err(ParseError::new(
                0,
                format!("property `{property}` has no value aliases"),
            ));
        }
    }

    Ok(aliases)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    line: usize,
    message: String,
}

impl ParseError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            formatter.write_str(&self.message)
        } else {
            write!(formatter, "line {}: {}", self.line, self.message)
        }
    }
}

impl Error for ParseError {}

#[cfg(test)]
mod tests {
    use crate::property_value_aliases::parse;

    const VALID: &str = "\
bc ; L ; Left_To_Right
ccc; 0 ; NR ; Not_Reordered
ea ; Na ; Narrow
";

    #[test]
    fn parses_the_supported_property_shapes() {
        let aliases = parse(VALID).unwrap();

        assert_eq!(aliases.len(), 3);
        assert_eq!(aliases[0].canonical(), "L");
        assert_eq!(aliases[0].abbreviation(), "L");
        assert_eq!(aliases[0].long_name(), "Left_To_Right");
        assert!(aliases[0].matches("left_to_right"));
        assert_eq!(aliases[1].canonical(), "0");
        assert_eq!(aliases[1].abbreviation(), "NR");
        assert_eq!(aliases[1].long_name(), "Not_Reordered");
    }

    #[test]
    fn rejects_the_wrong_shape_with_a_physical_line_number() {
        let source = "bc ; L ; Left_To_Right\nccc ; 0 ; Not_Reordered\nea ; Na ; Narrow";

        assert_eq!(
            parse(source).unwrap_err().to_string(),
            "line 2: property `ccc` expects 4 fields, found 3"
        );
    }

    #[test]
    fn requires_every_supported_property() {
        let source = "bc ; L ; Left_To_Right\nccc; 0 ; NR ; Not_Reordered";

        assert_eq!(
            parse(source).unwrap_err().to_string(),
            "property `ea` has no value aliases"
        );
    }
}
