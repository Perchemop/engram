//! Cross-cutting integration tests: library lifecycle, persistence to disk,
//! scheduling over a simulated study career, and import/export round-trips.
//!
//! All tests are headless (no display, no network) and write only inside
//! `target/`-adjacent temp directories that they clean up.

use chrono::{DateTime, TimeZone, Utc};
use engram::domain::{
    generate_drafts, schedule, Grade, LearnPhase, Library, LibraryError, SchedParams,
    SchedulingState, Scope,
};
use engram::domain::{CardFilter, CardKind};

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap()
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("engram_it_{tag}_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn simulated_study_career_graduates_a_card() {
    let mut lib = Library::new("Career", now());
    let deck = lib.decks[0].id;
    let card = lib
        .add_card(
            deck,
            CardKind::Basic {
                front: "front".into(),
                back: "back".into(),
            },
            vec![],
            now(),
        )
        .unwrap();
    let params = SchedParams::default();
    let t = |minutes: i64| now() + chrono::Duration::minutes(minutes);

    // New → learning step 0.
    let state = lib.card(card).unwrap().sched.clone();
    let s1 = schedule(t(0), &params, &state, Grade::Good);
    assert_eq!(s1.phase, LearnPhase::Learning);
    // Wait for the step, answer again.
    let s2 = schedule(t(10), &params, &s1, Grade::Good);
    assert_eq!(s2.phase, LearnPhase::Review, "second Good graduates");

    // Record the same journey through the library to exercise the log.
    lib.record_review(card, Grade::Again, t(0)).unwrap();
    lib.record_review(card, Grade::Good, t(1)).unwrap();
    lib.record_review(card, Grade::Good, t(11)).unwrap();
    lib.record_review(card, Grade::Good, t(11 + 60 * 24))
        .unwrap();
    assert_eq!(lib.reviews.len(), 4);
    let final_state = &lib.card(card).unwrap().sched;
    assert_eq!(final_state.phase, LearnPhase::Review);
    assert!(final_state.interval_days > 1.0);
}

#[test]
fn lapse_cycle_round_trips_through_relearning() {
    let mut lib = Library::new("Lapse", now());
    let deck = lib.decks[0].id;
    let card = lib
        .add_card(
            deck,
            CardKind::Basic {
                front: "f".into(),
                back: "b".into(),
            },
            vec![],
            now(),
        )
        .unwrap();
    let params = SchedParams::default();
    let base = SchedulingState {
        phase: LearnPhase::Review,
        step: 0,
        ease: 2.5,
        interval_days: 50.0,
        due: now(),
        reps: 10,
        lapses: 0,
    };
    // Apply directly for determinism, then mirror through the library.
    let after_again = schedule(now(), &params, &base, Grade::Again);
    assert_eq!(after_again.phase, LearnPhase::Relearning);
    let after_good = schedule(now(), &params, &after_again, Grade::Good);
    assert_eq!(after_good.phase, LearnPhase::Review);
    assert!(
        after_good.interval_days < base.interval_days,
        "a lapsed card restarts from the graduating interval"
    );

    // Library path logs both reviews with interval bookkeeping.
    lib.card_mut(card).unwrap().sched = base.clone();
    let out1 = lib.record_review(card, Grade::Again, now()).unwrap();
    assert_eq!(out1.log.interval_before_days, 50.0);
    let out2 = lib.record_review(card, Grade::Good, now()).unwrap();
    assert!(out2.log.interval_after_days > 0.0);
    assert_eq!(lib.reviews.len(), 2);
}

#[test]
fn daily_new_limit_is_respected_across_sessions() {
    let mut lib = Library::new("Limits", now());
    lib.settings.daily_new_limit = 1;
    let deck = lib.decks[0].id;
    for i in 0..5 {
        lib.add_card(
            deck,
            CardKind::Basic {
                front: format!("card {i}"),
                back: "b".into(),
            },
            vec![],
            now(),
        )
        .unwrap();
    }
    // Session 1: exactly one new card.
    let s1 = lib.build_session(Scope::All, now()).unwrap();
    assert_eq!(s1.total(), 1);
    // Answering it moves it to learning; ten minutes later the step is due,
    // so the next session serves that card plus one fresh new card.
    let (cid, _) = s1.current().unwrap();
    let later = now() + chrono::Duration::minutes(10);
    lib.record_review(cid, Grade::Good, later).unwrap();
    let s2 = lib.build_session(Scope::All, later).unwrap();
    assert_eq!(s2.total(), 2); // 1 learning (due) + 1 new
}

#[test]
fn persistence_roundtrip_on_disk() {
    let dir = temp_dir("persistence");
    let path = dir.join("library.engram.json");
    let mut lib = Library::new("Disk", now());
    let deck = lib.decks[0].id;
    let sub = lib.new_deck(Some(deck), "Sub", None, now()).unwrap();
    let card = lib
        .add_card(
            sub,
            CardKind::Cloze {
                text: "The {{c1::heart}} pumps blood".into(),
            },
            vec!["bio".into()],
            now(),
        )
        .unwrap();
    lib.record_review(card, Grade::Good, now()).unwrap();

    engram::data::save_library(&lib, &path).unwrap();
    let loaded = engram::data::load_library(&path).unwrap().unwrap();
    assert_eq!(loaded, lib);

    // Search still works on the loaded copy.
    let hits = loaded.search(&CardFilter {
        query: "heart".into(),
        ..CardFilter::default()
    });
    assert_eq!(hits, vec![card]);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn csv_roundtrip_between_libraries() {
    let mut source = Library::new("Source", now());
    let sdeck = source.decks[0].id;
    source.rename_deck(sdeck, "Exported").unwrap();
    source
        .add_card(
            sdeck,
            CardKind::Basic {
                front: "What is 2+2?".into(),
                back: "4".into(),
            },
            vec!["math".into()],
            now(),
        )
        .unwrap();
    source
        .add_card(
            sdeck,
            CardKind::Cloze {
                text: "{{c1::Iron}} rusts".into(),
            },
            vec![],
            now(),
        )
        .unwrap();

    let dir = temp_dir("csv");
    let path = dir.join("export.csv");
    let n = engram::data::export_csv(&path, &source, Scope::All).unwrap();
    assert_eq!(n, 2);

    let mut target = Library::new("Target", now());
    let tdeck = target.decks[0].id;
    let imported = engram::data::import_csv(&path, &mut target, tdeck, now()).unwrap();
    assert_eq!(imported, 2);
    assert_eq!(target.cards.len(), 2);
    assert!(target
        .cards
        .iter()
        .any(|c| matches!(c.kind, CardKind::Cloze { .. })));
    assert!(target.cards.iter().any(|c| c.tags.contains(&"math".into())));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn generator_output_lands_in_the_library() {
    let text = "ATP: the energy currency of the cell\n\
                Photosynthesis is the process plants use to make sugar";
    let drafts = generate_drafts(text);
    assert_eq!(drafts.len(), 2);

    let mut lib = Library::new("Gen", now());
    let deck = lib.decks[0].id;
    for d in &drafts {
        let kind = match d.kind {
            engram::domain::DraftKind::Cloze => CardKind::Cloze {
                text: d.front.clone(),
            },
            engram::domain::DraftKind::Basic => CardKind::Basic {
                front: d.front.clone(),
                back: d.back.clone(),
            },
        };
        lib.add_card(deck, kind, vec![], now()).unwrap();
    }
    assert_eq!(lib.cards.len(), 2);
    // The cloze card is directly studyable: prompt blanks the subject.
    let cloze = lib
        .cards
        .iter()
        .find(|c| matches!(c.kind, CardKind::Cloze { .. }))
        .unwrap();
    assert!(cloze.kind.prompt().contains('['));
}

#[test]
fn full_session_flow_end_to_end() {
    let mut lib = Library::new("Flow", now());
    let deck = lib.decks[0].id;
    for i in 0..3 {
        lib.add_card(
            deck,
            CardKind::Basic {
                front: format!("q{i}"),
                back: format!("a{i}"),
            },
            vec![],
            now(),
        )
        .unwrap();
    }
    let mut session = lib.build_session(Scope::All, now()).unwrap();
    let mut graded = 0;
    while let Some((cid, _)) = session.current() {
        session.answer(Grade::Good, now() + chrono::Duration::seconds(graded * 5));
        lib.record_review(cid, Grade::Good, now()).unwrap();
        graded += 1;
    }
    assert!(session.finished);
    assert_eq!(graded, 3);
    let summary = session.summary();
    assert_eq!(summary.total, 3);
    assert!((summary.retention() - 1.0).abs() < 1e-9);
    assert_eq!(lib.reviews.len(), 3);
}

#[test]
fn deleting_everything_leaves_a_usable_library() {
    let mut lib = Library::new("Wipe", now());
    let deck = lib.decks[0].id;
    let card = lib
        .add_card(
            deck,
            CardKind::Basic {
                front: "f".into(),
                back: "b".into(),
            },
            vec![],
            now(),
        )
        .unwrap();
    lib.record_review(card, Grade::Good, now()).unwrap();
    lib.delete_card(card).unwrap();
    lib.delete_deck(deck).unwrap();
    assert!(lib.cards.is_empty());
    assert!(lib.decks.is_empty());
    assert!(lib.reviews.is_empty());
    // Still usable: create a fresh deck and card.
    let new_deck = lib.new_deck(None, "Fresh", None, now()).unwrap();
    assert!(lib
        .add_card(
            new_deck,
            CardKind::Basic {
                front: "f".into(),
                back: "b".into()
            },
            vec![],
            now()
        )
        .is_ok());
}

#[test]
fn moving_a_deck_between_parents_updates_scope_counts() {
    let mut lib = Library::new("Move", now());
    let a = lib.new_deck(None, "A", None, now()).unwrap();
    let b = lib.new_deck(None, "B", None, now()).unwrap();
    let child = lib.new_deck(Some(a), "Child", None, now()).unwrap();
    lib.add_card(
        child,
        CardKind::Basic {
            front: "f".into(),
            back: "b".into(),
        },
        vec![],
        now(),
    )
    .unwrap();
    assert_eq!(lib.cards_in_scope(Scope::Deck(a)).len(), 1);
    assert_eq!(lib.cards_in_scope(Scope::Deck(b)).len(), 0);
    lib.move_deck(child, Some(b)).unwrap();
    assert_eq!(lib.cards_in_scope(Scope::Deck(a)).len(), 0);
    assert_eq!(lib.cards_in_scope(Scope::Deck(b)).len(), 1);
    assert_eq!(lib.deck_path(child).unwrap(), vec!["B", "Child"]);
}

#[test]
fn nothing_to_study_error_is_distinct() {
    let lib = Library::new("Empty", now());
    assert!(matches!(
        lib.build_session(Scope::All, now()),
        Err(LibraryError::NothingToStudy)
    ));
}
