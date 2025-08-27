use chrono::{Datelike, Duration, NaiveDate, Utc, Weekday};
use sporlcli::types::{Album, AlbumArtist, ReleaseTableRow};
use sporlcli::utils::*;
use std::collections::BTreeSet;

// Helper function to create a test album
fn create_test_album(id: &str, name: &str, release_date: &str, artist_name: &str) -> Album {
    Album {
        id: id.to_string(),
        name: name.to_string(),
        release_date: release_date.to_string(),
        release_date_precision: "day".to_string(),
        album_type: "album".to_string(),
        artists: vec![AlbumArtist {
            id: format!("{}_artist_id", id),
            name: artist_name.to_string(),
        }],
    }
}

// Helper function to create a test release table row
fn create_test_release_row(date: &str, name: &str, artists: &str) -> ReleaseTableRow {
    ReleaseTableRow {
        date: date.to_string(),
        name: name.to_string(),
        artists: artists.to_string(),
    }
}

#[test]
fn test_generate_code_verifier() {
    let verifier = generate_code_verifier();

    // Should be exactly 128 characters
    assert_eq!(verifier.len(), 128);

    // Should contain only alphanumeric characters
    assert!(verifier.chars().all(|c| c.is_ascii_alphanumeric()));

    // Two generated verifiers should be different
    let verifier2 = generate_code_verifier();
    assert_ne!(verifier, verifier2);
}

#[test]
fn test_generate_code_challenge() {
    let verifier = "test_verifier_123";
    let challenge = generate_code_challenge(verifier);

    // Should not be empty
    assert!(!challenge.is_empty());

    // Should be deterministic - same input produces same output
    let challenge2 = generate_code_challenge(verifier);
    assert_eq!(challenge, challenge2);

    // Different input should produce different output
    let challenge3 = generate_code_challenge("different_verifier");
    assert_ne!(challenge, challenge3);

    // Should be base64-encoded (URL-safe, no padding)
    assert!(
        challenge
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    );
}

#[test]
fn test_get_release_week_number() {
    // Test with January 1st (should handle year boundary correctly)
    let jan1 = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(); // Sunday
    let week_num = get_release_week_number(jan1);
    assert!(week_num >= 1);

    // Test with a date in the middle of the year
    let mid_year = NaiveDate::from_ymd_opt(2023, 6, 15).unwrap();
    let week_num = get_release_week_number(mid_year);
    assert!(week_num > 1 && week_num <= 53);

    // Test consistency - same week should have same number
    let same_week_date = NaiveDate::from_ymd_opt(2023, 6, 16).unwrap();
    assert_eq!(
        get_release_week_number(mid_year),
        get_release_week_number(same_week_date)
    );
}

#[test]
fn test_build_week() {
    let test_date = NaiveDate::from_ymd_opt(2023, 10, 17).unwrap(); // Tuesday
    let week = build_week(test_date);

    // Should have 7 dates
    assert_eq!(week.dates.len(), 7);

    // First date should be a Saturday
    assert_eq!(week.dates[0].weekday(), Weekday::Sat);

    // Last date should be a Friday
    assert_eq!(week.dates[6].weekday(), Weekday::Fri);

    // Dates should be consecutive
    for i in 1..7 {
        assert_eq!(week.dates[i], week.dates[i - 1] + Duration::days(1));
    }

    // Week number should be positive
    assert!(week.week >= 1);
}

#[test]
fn test_get_custom_week_range() {
    let test_date = NaiveDate::from_ymd_opt(2023, 10, 17).unwrap(); // Tuesday
    let weeks = get_custom_week_range(test_date, 3);

    // Should return 3 weeks (since Tuesday is not Friday, current week is skipped)
    assert_eq!(weeks.len(), 4);

    // Each week should have 7 dates
    for week in &weeks {
        assert_eq!(week.dates.len(), 7);
        assert_eq!(week.dates[0].weekday(), Weekday::Sat);
        assert_eq!(week.dates[6].weekday(), Weekday::Fri);
    }

    // Test with Friday (should include current week)
    let friday = NaiveDate::from_ymd_opt(2023, 10, 20).unwrap(); // Friday
    let weeks_friday = get_custom_week_range(friday, 2);
    assert_eq!(weeks_friday.len(), 3); // 2 previous + current week
}

#[test]
fn test_remove_duplicate_albums() {
    let mut albums = vec![
        create_test_album("id1", "Album 1", "2023-10-01", "Artist A"),
        create_test_album("id2", "Album 2", "2023-10-02", "Artist B"),
        create_test_album("id1", "Album 1 Duplicate", "2023-10-01", "Artist A"), // Duplicate
        create_test_album("id3", "Album 3", "2023-10-03", "Artist C"),
    ];

    remove_duplicate_albums(&mut albums);

    // Should have 3 unique albums
    assert_eq!(albums.len(), 3);

    // Should contain the first occurrence of each unique ID
    let ids: Vec<&String> = albums.iter().map(|a| &a.id).collect();
    assert_eq!(ids, vec!["id1", "id2", "id3"]);
}

#[test]
fn test_sort_release_table_rows() {
    let mut rows = vec![
        create_test_release_row("2023-10-01", "Album A", "Artist Z"),
        create_test_release_row("2023-10-03", "Album C", "Artist A"),
        create_test_release_row("2023-10-01", "Album B", "Artist A"), // Same date, different artist
        create_test_release_row("2023-10-02", "Album D", "Artist B"),
    ];

    sort_release_table_rows(&mut rows);

    // Should be sorted by date descending, then by artist ascending
    assert_eq!(rows[0].date, "2023-10-03"); // Most recent
    assert_eq!(rows[1].date, "2023-10-02");
    assert_eq!(rows[2].date, "2023-10-01");
    assert_eq!(rows[2].artists, "Artist A"); // Earlier alphabetically
    assert_eq!(rows[3].date, "2023-10-01");
    assert_eq!(rows[3].artists, "Artist Z"); // Later alphabetically
}

#[test]
fn test_get_date_from_string() {
    // Test valid date string
    let valid_date = get_date_from_string(Some("2023-10-17".to_string()));
    let expected = NaiveDate::from_ymd_opt(2023, 10, 17).unwrap();
    assert_eq!(valid_date, expected);

    // Test None input (should return current date)
    let current_date = get_date_from_string(None);
    let today = Utc::now().date_naive();
    assert_eq!(current_date, today);

    // Test invalid date string (should return current date)
    let invalid_date = get_date_from_string(Some("invalid-date".to_string()));
    let today = Utc::now().date_naive();
    assert_eq!(invalid_date, today);
}

#[test]
fn test_release_kind_display() {
    assert_eq!(ReleaseKind::Album.to_string(), "album");
    assert_eq!(ReleaseKind::Single.to_string(), "single");
    assert_eq!(ReleaseKind::AppearsOn.to_string(), "appears_on");
    assert_eq!(ReleaseKind::Compilation.to_string(), "compilation");
}

#[test]
fn test_release_kinds_default() {
    let default_kinds = ReleaseKinds::default();
    let collected: Vec<ReleaseKind> = default_kinds.iter().collect();
    assert_eq!(collected, vec![ReleaseKind::Album]);
}

#[test]
fn test_release_kinds_display() {
    // Test empty set (shouldn't happen in practice, but test the edge case)
    let empty_kinds = ReleaseKinds(BTreeSet::new());
    assert_eq!(empty_kinds.to_string(), "");

    // Test single kind
    let mut set = BTreeSet::new();
    set.insert(ReleaseKind::Album);
    let single_kind = ReleaseKinds(set);
    assert_eq!(single_kind.to_string(), "album");

    // Test multiple kinds (should be sorted)
    let mut set = BTreeSet::new();
    set.insert(ReleaseKind::Single);
    set.insert(ReleaseKind::Album);
    set.insert(ReleaseKind::Compilation);
    let multi_kinds = ReleaseKinds(set);
    assert_eq!(multi_kinds.to_string(), "album,single,compilation");
}

#[test]
fn test_parse_release_kinds_valid_inputs() {
    // Test single kind
    let result = parse_release_kinds("album").unwrap();
    let kinds: Vec<ReleaseKind> = result.iter().collect();
    assert_eq!(kinds, vec![ReleaseKind::Album]);

    // Test multiple kinds
    let result = parse_release_kinds("album,single").unwrap();
    let kinds: Vec<ReleaseKind> = result.iter().collect();
    assert_eq!(kinds, vec![ReleaseKind::Album, ReleaseKind::Single]);

    // Test "all" keyword
    let result = parse_release_kinds("all").unwrap();
    let kinds: Vec<ReleaseKind> = result.iter().collect();
    assert_eq!(kinds.len(), 4);
    assert!(kinds.contains(&ReleaseKind::Album));
    assert!(kinds.contains(&ReleaseKind::Single));
    assert!(kinds.contains(&ReleaseKind::AppearsOn));
    assert!(kinds.contains(&ReleaseKind::Compilation));

    // Test with spaces and hyphens
    let result = parse_release_kinds("album, appears-on").unwrap();
    let kinds: Vec<ReleaseKind> = result.iter().collect();
    assert_eq!(kinds, vec![ReleaseKind::Album, ReleaseKind::AppearsOn]);

    // Test case insensitivity
    let result = parse_release_kinds("ALBUM,Single").unwrap();
    let kinds: Vec<ReleaseKind> = result.iter().collect();
    assert_eq!(kinds, vec![ReleaseKind::Album, ReleaseKind::Single]);
}

#[test]
fn test_parse_release_kinds_invalid_inputs() {
    // Test empty string
    let result = parse_release_kinds("");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));

    // Test whitespace only
    let result = parse_release_kinds("   ");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));

    // Test invalid kind
    let result = parse_release_kinds("invalid_kind");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("invalid value 'invalid_kind'"));

    // Test malformed input (empty segment)
    let result = parse_release_kinds("album,,single");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty segment"));

    // Test mixed valid and invalid
    let result = parse_release_kinds("album,invalid,single");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("invalid value 'invalid'"));
}

#[test]
fn test_parse_release_kinds_deduplication() {
    // Test that duplicates are removed
    let result = parse_release_kinds("album,album,single").unwrap();
    let kinds: Vec<ReleaseKind> = result.iter().collect();
    assert_eq!(kinds, vec![ReleaseKind::Album, ReleaseKind::Single]);
}

#[test]
fn test_release_kinds_iter() {
    let mut set = BTreeSet::new();
    set.insert(ReleaseKind::Single);
    set.insert(ReleaseKind::Album);
    let kinds = ReleaseKinds(set);

    let collected: Vec<ReleaseKind> = kinds.iter().collect();
    // Should be sorted due to BTreeSet
    assert_eq!(collected, vec![ReleaseKind::Album, ReleaseKind::Single]);
}

#[test]
fn test_release_kind_all_constant() {
    // Ensure ALL constant contains all variants
    assert_eq!(ReleaseKind::ALL.len(), 4);
    assert!(ReleaseKind::ALL.contains(&ReleaseKind::Album));
    assert!(ReleaseKind::ALL.contains(&ReleaseKind::Single));
    assert!(ReleaseKind::ALL.contains(&ReleaseKind::AppearsOn));
    assert!(ReleaseKind::ALL.contains(&ReleaseKind::Compilation));
}
