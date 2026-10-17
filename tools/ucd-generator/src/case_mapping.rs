use std::collections::{BTreeMap, BTreeSet};

use crate::ucd::{ParseError, parse_code_point};

// Uppercase, lowercase, and titlecase, in the order shown by the Inspector.
pub type CaseMappings = BTreeMap<u32, [Vec<u32>; 3]>;

pub fn full_folding(source: &str) -> Result<BTreeMap<u32, Vec<u32>>, ParseError> {
    let mut mappings = BTreeMap::new();
    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        let content = line.split('#').next().unwrap().trim();
        if content.is_empty() {
            continue;
        }
        let fields = content
            .strip_suffix(';')
            .unwrap_or(content)
            .split(';')
            .map(str::trim)
            .collect::<Vec<_>>();
        let [code_point, status, mapping] = fields.as_slice() else {
            return Err(ParseError::new(
                line_number,
                "expected code point, folding status, and mapping",
            ));
        };
        let code_point = parse_code_point(line_number, code_point)?;
        match *status {
            "C" | "F" => {
                let mapping = parse_mapping(line_number, mapping)?;
                if mappings.insert(code_point, mapping).is_some() {
                    return Err(ParseError::new(
                        line_number,
                        "duplicate default full case folding",
                    ));
                }
            }
            "S" | "T" => {}
            _ => return Err(ParseError::new(line_number, "unknown case folding status")),
        }
    }
    mappings.retain(|&code_point, mapping| mapping != &[code_point]);
    Ok(mappings)
}

pub fn full_mappings(mut simple: CaseMappings, source: &str) -> Result<CaseMappings, ParseError> {
    let mut seen = BTreeSet::new();
    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        let content = line.split('#').next().unwrap().trim();
        if content.is_empty() {
            continue;
        }
        let fields = content
            .strip_suffix(';')
            .unwrap_or(content)
            .split(';')
            .map(str::trim)
            .collect::<Vec<_>>();
        if !(4..=5).contains(&fields.len()) {
            return Err(ParseError::new(
                line_number,
                "expected code point, three mappings, and optional conditions",
            ));
        }
        let code_point = parse_code_point(line_number, fields[0])?;
        if fields
            .get(4)
            .is_some_and(|conditions| !conditions.is_empty())
        {
            continue;
        }
        if !seen.insert(code_point) {
            return Err(ParseError::new(
                line_number,
                "duplicate unconditional case mapping",
            ));
        }
        simple.insert(
            code_point,
            [
                parse_mapping(line_number, fields[3])?,
                parse_mapping(line_number, fields[1])?,
                parse_mapping(line_number, fields[2])?,
            ],
        );
    }
    Ok(simple)
}

pub fn parse_mapping(line: usize, source: &str) -> Result<Vec<u32>, ParseError> {
    let values = source
        .split_whitespace()
        .map(|value| {
            let code_point = parse_code_point(line, value)?;
            if char::from_u32(code_point).is_none() {
                return Err(ParseError::new(
                    line,
                    "case mapping target must be a Unicode scalar value",
                ));
            }
            Ok(code_point)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if values.is_empty() {
        return Err(ParseError::new(line, "case mapping must not be empty"));
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_full_folding_uses_common_and_full_entries_only() {
        let mappings = full_folding("0049; C; 0069;\n0049; T; 0131;\n00DF; F; 0073 0073;\n1E9E; S; 00DF;\n1E9E; F; 0073 0073;").unwrap();
        assert_eq!(mappings[&0x49], [0x69]);
        assert_eq!(mappings[&0xdf], [0x73, 0x73]);
        assert_eq!(mappings[&0x1e9e], [0x73, 0x73]);
        for source in [
            "0041; X; 0061;",
            "0041; C; D800;",
            "0041; C; 0061;\n0041; C; 0062;",
        ] {
            assert!(full_folding(source).is_err());
        }
    }

    #[test]
    fn full_mappings_override_simple_values_and_skip_language_and_context_rules() {
        let simple = BTreeMap::from([
            (0x130, [vec![0x130], vec![0x69], vec![0x130]]),
            (0x3a3, [vec![0x3a3], vec![0x3c3], vec![0x3a3]]),
        ]);
        let full = full_mappings(simple, "00DF; 00DF; 0053 0073; 0053 0053;\n0130; 0069 0307; 0130; 0130;\n0130; 0069; 0130; 0130; tr;\n03A3; 03C2; 03A3; 03A3; Final_Sigma;").unwrap();
        assert_eq!(
            full[&0xdf],
            [vec![0x53, 0x53], vec![0xdf], vec![0x53, 0x73]]
        );
        assert_eq!(full[&0x130][1], [0x69, 0x307]);
        assert_eq!(full[&0x3a3][1], [0x3c3]);
        for source in [
            "00DF; 00DF; 0053; D800;",
            "00DF; 00DF; 0053; ;",
            "00DF; 00DF; 0053; 0053;\n00DF; 00DF; 0053; 0053;",
        ] {
            assert!(full_mappings(BTreeMap::new(), source).is_err());
        }
    }
}
