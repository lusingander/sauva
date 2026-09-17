use fontdb::{Database, Family, Query};

pub fn default_font(database: &Database) -> Option<fontdb::ID> {
    database.query(&Query {
        families: &[Family::SansSerif],
        ..Query::default()
    })
}

pub fn fallback_font(
    database: &Database,
    character: char,
    needs_dotted_circle: bool,
) -> Option<fontdb::ID> {
    let mut candidates = database.faces().collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        fallback_rank(left)
            .cmp(&fallback_rank(right))
            .then_with(|| left.post_script_name.cmp(&right.post_script_name))
            .then_with(|| left.index.cmp(&right.index))
    });
    candidates
        .into_iter()
        .find(|face| supports(database, face.id, character, needs_dotted_circle))
        .map(|face| face.id)
}

fn fallback_rank(face: &fontdb::FaceInfo) -> (u8, u16) {
    let style = match face.style {
        fontdb::Style::Normal => 0,
        fontdb::Style::Italic | fontdb::Style::Oblique => 1,
    };
    (style, face.weight.0.abs_diff(fontdb::Weight::NORMAL.0))
}

pub fn supports(
    database: &Database,
    id: fontdb::ID,
    character: char,
    needs_dotted_circle: bool,
) -> bool {
    database
        .with_face_data(id, |data, index| {
            let Some(font) = swash::FontRef::from_index(data, index as usize) else {
                return false;
            };
            font.charmap().map(character) != 0
                && (!needs_dotted_circle || font.charmap().map('\u{25cc}') != 0)
        })
        .unwrap_or(false)
}
