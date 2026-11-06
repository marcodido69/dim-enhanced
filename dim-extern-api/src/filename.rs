pub use anitomy::Anitomy;
use anitomy::ElementCategory;
pub use torrent_name_parser::Metadata as TorrentMetadata;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Metadata {
    pub name: String,
    pub year: Option<i64>,
    pub season: Option<i64>,
    pub episode: Option<i64>,
    /// Id de TMDB embebido en el nombre del archivo, por ejemplo `{tmdb-12345}`.
    pub tmdb_id: Option<i64>,
    /// Id de IMDb embebido en el nombre del archivo, por ejemplo `{imdb-tt1234567}`.
    pub imdb_id: Option<String>,
}

pub trait FilenameMetadata {
    fn from_str(s: &str) -> Option<Metadata>;
}

impl FilenameMetadata for TorrentMetadata {
    fn from_str(s: &str) -> Option<Metadata> {
        let metadata = TorrentMetadata::from(s).ok()?;

        Some(Metadata {
            name: metadata.title().to_owned(),
            year: metadata.year().map(|x| x as i64),
            season: metadata.season().map(|x| x as i64),
            episode: metadata.episode().map(|x| x as i64),
            tmdb_id: None,
            imdb_id: None,
        })
    }
}

impl FilenameMetadata for Anitomy {
    fn from_str(s: &str) -> Option<Metadata> {
        let metadata = match Anitomy::new().parse(s) {
            Ok(v) | Err(v) => v,
        };

        Some(Metadata {
            name: metadata.get(ElementCategory::AnimeTitle)?.to_string(),
            year: metadata
                .get(ElementCategory::AnimeYear)
                .and_then(|x| x.parse().ok()),
            // If season isnt specified we assume season 1 here.
            season: metadata
                .get(ElementCategory::AnimeSeason)
                .and_then(|x| x.parse().ok())
                .or(Some(1)),
            episode: metadata
                .get(ElementCategory::EpisodeNumber)
                .and_then(|x| x.parse().ok()),
            tmdb_id: None,
            imdb_id: None,
        })
    }
}

/// A special filename metadata extractor that combines torrent_name_parser and anitomy which in
/// some cases is necessary. TNP is really good at extracting show titles but not season and
/// episode numbers. Anitomy excels at this. Here we combine the title extracted by TPN and the
/// season and episode number extracted by Anitomy.
pub struct CombinedExtractor;

impl FilenameMetadata for CombinedExtractor {
    fn from_str(s: &str) -> Option<Metadata> {
        let metadata_tnp = TorrentMetadata::from(s).ok()?;
        let metadata_anitomy = match Anitomy::new().parse(s) {
            Ok(v) | Err(v) => v,
        };

        Some(Metadata {
            name: metadata_tnp.title().to_owned(),
            year: metadata_tnp.year().map(|x| x as i64),
            // If season isnt specified we assume season 1 here as some releases only have a
            // episode number and no season number.
            season: metadata_anitomy
                .get(ElementCategory::AnimeSeason)
                .and_then(|x| x.parse().ok())
                .or(Some(1)),
            episode: metadata_anitomy
                .get(ElementCategory::EpisodeNumber)
                .and_then(|x| x.parse().ok()),
            tmdb_id: None,
            imdb_id: None,
        })
    }
}

/// Tags de release que no forman parte del titulo real.
const RELEASE_TAGS: &[&str] = &[
    "1080p", "720p", "2160p", "480p", "4k", "uhd", "hdr", "hdr10", "hdr10+", "dv", "dolby",
    "vision", "bluray", "blu-ray", "bdrip", "brrip", "bdremux", "remux", "webrip", "web-dl",
    "webdl", "web", "hdtv", "dvdrip", "dvd", "cam", "camrip", "ts", "tc", "hdrip", "uhdrip",
    "x264", "x265", "h264", "h265", "hevc", "avc", "10bit", "8bit", "8bits", "10bits", "aac",
    "ac3", "eac3", "dd", "ddp", "dd5", "dts", "dtshd", "dts-hd", "truehd", "atmos", "flac", "mp3",
    "opus", "5.1", "7.1", "2.0", "proper", "repack", "extended", "unrated", "directors", "cut",
    "imax", "limited", "internal", "multi", "subs", "subbed", "dubbed", "dual", "audio",
    "spanish", "latino", "castellano", "english", "ita", "fre", "ger", "complete", "retail",
    "rerip", "remastered", "remaster", "anniversary", "criterion", "edition", "uncut",
];

fn is_release_tag(token: &str) -> bool {
    RELEASE_TAGS.contains(&token)
}

/// Quita grupos entre corchetes, llaves y parentesis (ids, tags, notas).
fn strip_bracket_groups(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut depth: i32 = 0;

    for ch in input.chars() {
        match ch {
            '[' | '{' | '(' => depth += 1,
            ']' | '}' | ')' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ => {
                if depth == 0 {
                    out.push(ch);
                }
            }
        }
    }

    out
}

/// Ids embebidos en el nombre: `{tmdb-123}`, `[tmdbid-123]`, `{imdb-tt1234567}`.
fn extract_external_ids(input: &str) -> (Option<i64>, Option<String>) {
    let lower = input.to_lowercase();
    let mut tmdb_id = None;
    let mut imdb_id = None;

    for key in ["tmdbid", "tmdb"] {
        if let Some(pos) = lower.find(key) {
            let rest = &lower[pos + key.len()..];
            let rest = rest.trim_start_matches(|c: char| {
                c == '-' || c == '_' || c == ' ' || c == '{' || c == '[' || c == '('
            });
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();

            if let Ok(id) = digits.parse::<i64>() {
                if id > 0 {
                    tmdb_id = Some(id);
                    break;
                }
            }
        }
    }

    for key in ["imdbid", "imdb"] {
        if let Some(pos) = lower.find(key) {
            let rest = &lower[pos + key.len()..];
            let rest = rest.trim_start_matches(|c: char| {
                c == '-' || c == '_' || c == ' ' || c == '{' || c == '[' || c == '('
            });

            if let Some(rest) = rest.strip_prefix("tt") {
                let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if digits.len() >= 5 {
                    imdb_id = Some(format!("tt{digits}"));
                    break;
                }
            }
        }
    }

    (tmdb_id, imdb_id)
}

/// Anio de estreno. Prioriza anios entre parentesis; si no hay, acepta un anio
/// suelto solo cuando el nombre parece de release (tiene tags como 1080p/WEB-DL),
/// para no confundir titulos como "2001: A Space Odyssey".
fn year_from_token(token: &str) -> Option<i64> {
    let token = token.trim_matches(|c: char| !c.is_ascii_digit());

    if token.len() != 4 || !token.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let year = token.parse::<i64>().ok()?;

    if (1900..=2035).contains(&year) {
        Some(year)
    } else {
        None
    }
}

fn extract_year(input: &str) -> Option<i64> {
    let mut bracketed_year = None;
    let mut standalone_year = None;
    let mut has_release_tag = false;
    let mut depth: i32 = 0;
    let mut current = String::new();

    for ch in input.chars() {
        match ch {
            '(' | '[' | '{' => {
                if !current.trim().is_empty() {
                    if is_release_tag(&current.trim().to_lowercase()) {
                        has_release_tag = true;
                    }
                    if standalone_year.is_none() {
                        standalone_year = year_from_token(&current);
                    }
                }
                current.clear();
                depth += 1;
            }
            ')' | ']' | '}' => {
                if depth > 0 {
                    if let Some(year) = year_from_token(&current) {
                        bracketed_year = Some(year);
                    }
                    current.clear();
                    depth -= 1;
                }
            }
            c if depth > 0 => current.push(c),
            c if c == '.' || c == '_' || c.is_whitespace() => {
                if !current.trim().is_empty() {
                    if is_release_tag(&current.trim().to_lowercase()) {
                        has_release_tag = true;
                    }
                    if standalone_year.is_none() {
                        standalone_year = year_from_token(&current);
                    }
                }
                current.clear();
            }
            c => current.push(c),
        }
    }

    if !current.trim().is_empty() {
        if is_release_tag(&current.trim().to_lowercase()) {
            has_release_tag = true;
        }
        if standalone_year.is_none() {
            standalone_year = year_from_token(&current);
        }
    }

    bracketed_year.or(if has_release_tag { standalone_year } else { None })
}

/// Extrae el titulo limpio: sin ids, corchetes, tags de release ni el anio.
fn clean_release_name(input: &str, year: Option<i64>) -> String {
    let stripped = strip_bracket_groups(input);
    let mut tokens: Vec<&str> = Vec::new();

    for raw in stripped.split(|c: char| c == '.' || c == '_' || c.is_whitespace()) {
        let token = raw
            .trim_matches(|c: char| matches!(c, '-' | ':' | ';' | ',' | '\'' | '"' | '!' | '?'));

        if token.is_empty() {
            continue;
        }

        let lower = token.to_lowercase();

        if is_release_tag(&lower) {
            continue;
        }

        if let Some(year) = year {
            if lower == year.to_string() {
                continue;
            }
        }

        tokens.push(token);
    }

    tokens
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_digits(chars: &[(usize, char)], start: usize, max: usize) -> (String, usize) {
    let mut digits = String::new();
    let mut i = start;

    while i < chars.len() && chars[i].1.is_ascii_digit() && digits.len() < max {
        digits.push(chars[i].1);
        i += 1;
    }

    (digits, i)
}

fn skip_separators(chars: &[(usize, char)], mut i: usize) -> usize {
    while i < chars.len() && matches!(chars[i].1, ' ' | '-' | '_' | '.') {
        i += 1;
    }

    i
}

/// Busca patrones tipo `S01E02`, `1x02` o `Season 1 Episode 2`.
/// Devuelve `(temporada, episodio, indice de byte donde empieza el patron)`.
fn find_season_episode(input: &str) -> Option<(i64, i64, usize)> {
    let chars: Vec<(usize, char)> = input.char_indices().collect();
    let lower: Vec<char> = input.chars().map(|c| c.to_ascii_lowercase()).collect();
    let n = chars.len();

    let mut i = 0;
    while i < n {
        let c = lower[i];

        // S01E02 / S1E2
        if c == 's' && (i == 0 || !chars[i - 1].1.is_alphanumeric()) {
            let (season, mut j) = parse_digits(&chars, i + 1, 2);

            if !season.is_empty() {
                j = skip_separators(&chars, j);

                if j < n && lower[j] == 'e' {
                    let (episode, _) = parse_digits(&chars, j + 1, 3);

                    if !episode.is_empty() {
                        return Some((season.parse().ok()?, episode.parse().ok()?, chars[i].0));
                    }
                }
            }
        }

        // 1x02
        if c.is_ascii_digit() && (i == 0 || !chars[i - 1].1.is_ascii_digit()) {
            let (season, j) = parse_digits(&chars, i, 2);

            if !season.is_empty() && j < n && lower[j] == 'x' {
                let (episode, _) = parse_digits(&chars, j + 1, 3);

                if !episode.is_empty() && season.parse::<i64>().map(|x| x <= 30).unwrap_or(false) {
                    return Some((season.parse().ok()?, episode.parse().ok()?, chars[i].0));
                }
            }
        }

        // Season 1 Episode 2
        if lower[i..].starts_with(&['s', 'e', 'a', 's', 'o', 'n']) {
            let mut j = skip_separators(&chars, i + 6);
            let (season, next) = parse_digits(&chars, j, 2);
            j = skip_separators(&chars, next);

            if !season.is_empty() && lower[j..].starts_with(&['e', 'p', 'i', 's', 'o', 'd', 'e']) {
                let k = skip_separators(&chars, j + 7);
                let (episode, _) = parse_digits(&chars, k, 3);

                if !episode.is_empty() {
                    return Some((season.parse().ok()?, episode.parse().ok()?, chars[i].0));
                }
            }
        }

        i += 1;
    }

    None
}

/// Extractor inspirado en las convenciones de nombres de Jellyfin: titulos limpios,
/// temporada/episodio en multiples formatos e ids de TMDB/IMDb embebidos.
pub struct JellyfinExtractor;

impl FilenameMetadata for JellyfinExtractor {
    fn from_str(s: &str) -> Option<Metadata> {
        let (tmdb_id, imdb_id) = extract_external_ids(s);
        let year = extract_year(s);
        let season_episode = find_season_episode(s);

        let name = match season_episode {
            Some((_, _, pos)) => clean_release_name(&s[..pos], year),
            None => clean_release_name(s, year),
        };

        let name = if name.is_empty() {
            s.trim().to_string()
        } else {
            name
        };

        Some(Metadata {
            name,
            year,
            season: season_episode.map(|x| x.0),
            episode: season_episode.map(|x| x.1),
            tmdb_id,
            imdb_id,
        })
    }
}

/// Normaliza un titulo para comparaciones: minusculas, solo alfanumerico y espacios simples.
pub fn normalize_title(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Similitud simple entre el titulo esperado y el titulo de un resultado.
pub fn title_similarity(expected: &str, actual: &str) -> i64 {
    let a = normalize_title(expected);
    let b = normalize_title(actual);

    if a.is_empty() || b.is_empty() {
        return 0;
    }

    if a == b {
        return 100;
    }

    if a.contains(&b) || b.contains(&a) {
        return 70;
    }

    let expected_words: Vec<&str> = a.split_whitespace().collect();
    let actual_words: Vec<&str> = b.split_whitespace().collect();
    let common = expected_words
        .iter()
        .filter(|word| actual_words.contains(word))
        .count();

    if common == 0 {
        return 0;
    }

    (common as f64 / expected_words.len() as f64 * 60.0) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_movie_with_year_and_accents() {
        let metadata = JellyfinExtractor::from_str("Ãƒâ€šÃ‚Â¡No vayas a colgar! (2016)").unwrap();

        assert_eq!(metadata.name, "Ãƒâ€šÃ‚Â¡No vayas a colgar");
        assert_eq!(metadata.year, Some(2016));
        assert_eq!(metadata.season, None);
        assert_eq!(metadata.episode, None);
    }

    #[test]
    fn parses_scene_movie_name() {
        let metadata =
            JellyfinExtractor::from_str("Spider-Man.No.Way.Home.2021.1080p.WEB-DL.x264").unwrap();

        assert_eq!(metadata.name, "Spider-Man No Way Home");
        assert_eq!(metadata.year, Some(2021));
    }

    #[test]
    fn parses_movie_with_tmdb_id() {
        let metadata = JellyfinExtractor::from_str("Dune.Part.Two.2024.2160p {tmdb-693134}").unwrap();

        assert_eq!(metadata.name, "Dune Part Two");
        assert_eq!(metadata.year, Some(2024));
        assert_eq!(metadata.tmdb_id, Some(693134));
    }

    #[test]
    fn parses_movie_with_imdb_id() {
        let metadata =
            JellyfinExtractor::from_str("The Batman (2022) [imdbid-tt1877830]").unwrap();

        assert_eq!(metadata.name, "The Batman");
        assert_eq!(metadata.year, Some(2022));
        assert_eq!(metadata.imdb_id.as_deref(), Some("tt1877830"));
    }

    #[test]
    fn does_not_treat_title_number_as_year() {
        let metadata = JellyfinExtractor::from_str("Blade Runner 2049").unwrap();

        assert_eq!(metadata.name, "Blade Runner 2049");
        assert_eq!(metadata.year, None);
    }

    #[test]
    fn parses_episode_sxxexx() {
        let metadata =
            JellyfinExtractor::from_str("The Office - S02E01 - The Dundies [1080p]").unwrap();

        assert_eq!(metadata.name, "The Office");
        assert_eq!(metadata.season, Some(2));
        assert_eq!(metadata.episode, Some(1));
    }

    #[test]
    fn parses_multi_episode_takes_first() {
        let metadata = JellyfinExtractor::from_str("Breaking Bad S01E02E03 1080p").unwrap();

        assert_eq!(metadata.name, "Breaking Bad");
        assert_eq!(metadata.season, Some(1));
        assert_eq!(metadata.episode, Some(2));
    }

    #[test]
    fn parses_episode_1x02() {
        let metadata = JellyfinExtractor::from_str("Show 1x02 Pilot").unwrap();

        assert_eq!(metadata.name, "Show");
        assert_eq!(metadata.season, Some(1));
        assert_eq!(metadata.episode, Some(2));
    }

    #[test]
    fn parses_episode_season_episode_words() {
        let metadata = JellyfinExtractor::from_str("Star Trek Season 2 Episode 5").unwrap();

        assert_eq!(metadata.name, "Star Trek");
        assert_eq!(metadata.season, Some(2));
        assert_eq!(metadata.episode, Some(5));
    }

    #[test]
    fn parses_show_with_year() {
        let metadata =
            JellyfinExtractor::from_str("The Office (2005) S02E01 The Dundies 1080p").unwrap();

        assert_eq!(metadata.name, "The Office");
        assert_eq!(metadata.season, Some(2));
        assert_eq!(metadata.episode, Some(1));
    }

    #[test]
    fn title_similarity_prefers_normalized_match() {
        assert_eq!(title_similarity("Spider-Man No Way Home", "Spider Man No Way Home"), 100);
        assert!(title_similarity("The Office", "The Office (US)") >= 70);
        assert!(title_similarity("The Office", "Breaking Bad") < 30);
    }

    #[test]
    fn mixed_case_extensionless_ids() {
        let metadata =
            JellyfinExtractor::from_str("Movie.2020.{TMDB-12345}.1080p").unwrap();

        assert_eq!(metadata.tmdb_id, Some(12345));
        assert_eq!(metadata.year, Some(2020));
    }
}



