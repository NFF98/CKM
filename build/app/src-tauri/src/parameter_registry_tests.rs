use super::*;
use crate::{goal_draft::{ConfirmGoalInput, GoalStore},local_store::{IncomingDrop, LocalStore}};

fn private_content(s: &str) -> IncomingDrop {
    IncomingDrop {kind:"text".into(),source:"text".into(),
        text:Some(s.into()),base64:None,file_name:None,mime_type:None}
}

fn setup() -> (tempfile::TempDir,String,String) {
    let dir=tempfile::tempdir().unwrap();
    let asset=LocalStore::open(dir.path()).unwrap()
        .save(private_content("パクる JLPT N2")).unwrap();
    let goal=GoalStore::open(dir.path()).unwrap()
        .confirm(ConfirmGoalInput {asset_id:asset.id.clone(),
            goal_text:"真人 N2 學伴".into(), confirmation_action:"confirm_goal".into(),
            expected_revision:0}).unwrap();
    (dir,asset.id,goal.id)
}

fn proposal(draft_id:&str,alias:&str,value:Option<&str>,version:i64)->SaveProposalInput {
    SaveProposalInput {draft_id:draft_id.into(),expected_draft_revision:1,
        concept_alias:alias.into(),value:value.map(str::to_owned),scope:"intent".into(),
        expiry_policy:"intent_lifecycle".into(),expires_at_ms:None,expected_version:version}
}

fn confirmation(draft_id:&str,id:&str,version:i64)->ConfirmParameterInput {
    ConfirmParameterInput {draft_id:draft_id.into(),proposal_id:id.into(),
        expected_version:version,confirmation_action:"confirm_parameter".into()}
}

#[test]
fn t04_aliases_n2_jlpt_n2_and_chinese_same_single_concept() {
    let (dir,_,draft)=setup();
    let store=ParameterStore::open(dir.path()).unwrap();
    let first=store.save(proposal(&draft,"N2",None,0)).unwrap();
    assert_eq!(first.concept_id.as_deref(),Some("study.jlpt_level"));
    assert_eq!(first.value_type,"enum");
    assert_eq!(first.value_json.as_deref(),Some(r#""N2""#));
    assert_eq!(first.state,"OBSERVED");
    assert_eq!(first.visibility,"private");
    assert!(!first.share_consent);
    let second=store.save(proposal(&draft,"JLPT N2",None,1)).unwrap();
    let third=store.save(proposal(&draft,"日語二級",None,2)).unwrap();
    assert_eq!(first.id,second.id);
    assert_eq!(first.id,third.id);
    assert_eq!(third.version,3);
    assert_eq!(store.list(&draft).unwrap().len(),1);
    assert_eq!(registry().len(),1); // T-01 remaining whitelist not invented.
}

#[test]
fn t01_ai_cannot_forge_confirmation_or_source_consent() {
    let (dir,_,draft)=setup();
    let store=ParameterStore::open(dir.path()).unwrap();
    for bad in [
        serde_json::json!({"draftId":draft,"expectedDraftRevision":1,
           "conceptAlias":"N2","scope":"intent","expiryPolicy":"intent_lifecycle",
           "expectedVersion":0,"state":"CONFIRMED"}),
        serde_json::json!({"draftId":draft,"expectedDraftRevision":1,
           "conceptAlias":"N2","scope":"intent","expiryPolicy":"intent_lifecycle",
           "expectedVersion":0,"source":"model_inferred"}),
        serde_json::json!({"draftId":draft,"expectedDraftRevision":1,
           "conceptAlias":"N2","scope":"intent","expiryPolicy":"intent_lifecycle",
           "expectedVersion":0,"confirmedEventId":"fake"}),
        serde_json::json!({"draftId":draft,"expectedDraftRevision":1,
           "conceptAlias":"N2","scope":"intent","expiryPolicy":"intent_lifecycle",
           "expectedVersion":0,"visibility":"published","consent":true}),
    ] {
        assert!(serde_json::from_value::<SaveProposalInput>(bad).is_err());
    }
    let first=store.save(proposal(&draft,"N2",None,0)).unwrap();
    let mut fake=confirmation(&draft,&first.id,first.version);
    fake.confirmation_action="model_claimed_confirmed".into();
    assert_eq!(store.confirm(fake).unwrap_err(),"PARAMETER_CONFIRMATION_REQUIRED");
    assert_eq!(store.list(&draft).unwrap()[0].state,"OBSERVED");
}

#[test]
fn t01_confirm_only_new_rust_event_versioned_private() {
    let (dir,_,draft)=setup();
    let store=ParameterStore::open(dir.path()).unwrap();
    let first=store.save(proposal(&draft,"study.jlpt_level",Some("JLPT N2"),0)).unwrap();
    let saved=store.confirm(confirmation(&draft,&first.id,1)).unwrap();
    assert_eq!(saved.state,"CONFIRMED");
    assert_eq!(saved.version,2);
    assert_eq!(saved.scope,"intent");
    assert_eq!(saved.source,"user_explicit");
    assert_eq!(saved.expiry_policy,"intent_lifecycle");
    assert_eq!(saved.visibility,"private");
    assert!(!saved.share_consent);
    assert!(saved.confirmed_event_id.is_some());
    assert_ne!(saved.confirmed_event_id,Some(saved.evidence_ref.clone()));
    assert_eq!(store.confirm(confirmation(&draft,&saved.id,1)).unwrap_err(),
        "PARAMETER_VERSION_CONFLICT");
    let conn=store.connect().unwrap();
    let event: (String,String,i64)=conn.query_row(
        "SELECT action,source,version FROM local_parameter_confirmation_events
         WHERE id=?1",params![saved.confirmed_event_id],
        |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!(event,("explicit_parameter_confirmation".into(),
        "user_explicit".into(),2));
    drop(store);
    let reopened=ParameterStore::open(dir.path()).unwrap();
    assert_eq!(reopened.list(&draft).unwrap()[0].state,"CONFIRMED");
}

#[test]
fn t02_rust_and_sqlite_reject_missing_or_unauthorized_metadata() {
    let (dir,_,draft)=setup();
    let store=ParameterStore::open(dir.path()).unwrap();
    let proposal=store.save(proposal(&draft,"N2",None,0)).unwrap();
    let conn=store.connect().unwrap();
    for sql in [
        "UPDATE local_parameter_drafts SET source=NULL",
        "UPDATE local_parameter_drafts SET source='model_inferred'",
        "UPDATE local_parameter_drafts SET scope=NULL",
        "UPDATE local_parameter_drafts SET scope='profile'",
        "UPDATE local_parameter_drafts SET expiry_policy=NULL",
        "UPDATE local_parameter_drafts SET expiry_policy='persistent_review'",
        "UPDATE local_parameter_drafts SET visibility='published'",
        "UPDATE local_parameter_drafts SET share_consent=1",
        "UPDATE local_parameter_drafts SET version=0",
        "UPDATE local_parameter_drafts SET state='CONFIRMED'",
        "UPDATE local_parameter_drafts SET concept_id='unknown.new_parameter'",
        "UPDATE local_parameter_drafts SET value_json='\"N1\"'",
        "UPDATE local_parameter_drafts SET value_json=NULL",
        "UPDATE local_parameter_drafts SET expiry_policy='fixed',expires_at_ms=NULL",
    ] {
        assert!(conn.execute(sql,[]).is_err(),"sqlite must block {sql}");
    }
    assert_eq!(store.list(&draft).unwrap()[0].id,proposal.id);
}

#[test]
fn t02_missing_provenance_scope_expiry_is_rejected_by_rust_deserialization() {
    let (_,_,draft)=setup();
    let required=serde_json::json!({
        "draftId":draft,
        "expectedDraftRevision":1,
        "conceptAlias":"N2",
        "scope":"intent",
        "expiryPolicy":"intent_lifecycle",
        "expectedVersion":0
    });
    for missing in ["draftId","scope","expiryPolicy","expectedVersion"] {
        let mut object=required.clone();
        object.as_object_mut().unwrap().remove(missing);
        assert!(serde_json::from_value::<SaveProposalInput>(object).is_err(),
                "missing {missing} must be rejected");
    }
    let mut fake_source=required;
    fake_source["source"]=serde_json::json!("model_inferred");
    assert!(serde_json::from_value::<SaveProposalInput>(fake_source).is_err());
}

#[test]
fn t03_fixed_date_cannot_be_inferred_or_made_recurring() {
    let (dir,_,draft)=setup();
    let store=ParameterStore::open(dir.path()).unwrap();
    let mut sat=proposal(&draft,"週六有空",None,0);
    sat.expiry_policy="fixed".into();
    assert_eq!(store.save(sat).unwrap_err(),"INVALID_EXPIRY");
    let mut sat=proposal(&draft,"週六有空",None,0);
    sat.expiry_policy="fixed".into();
    sat.expires_at_ms=Some(clock_ms().unwrap()+86_400_000);
    let saved=store.save(sat).unwrap();
    assert_eq!(saved.state,"UNKNOWN");
    assert!(saved.concept_id.is_none());
    assert_eq!(saved.expiry_policy,"fixed");
    assert_eq!(saved.expires_at_ms,Some(store.list(&draft).unwrap()[0].expires_at_ms.unwrap()));
    assert_eq!(store.confirm(confirmation(&draft,&saved.id,saved.version)).unwrap_err(),
        "PARAMETER_UNAPPROVED_CONCEPT");
    let mut invalid=proposal(&draft,"N2",None,0);
    invalid.expiry_policy="intent_lifecycle".into();
    invalid.expires_at_ms=Some(clock_ms().unwrap()+86_400_000);
    assert_eq!(store.save(invalid).unwrap_err(),"INVALID_EXPIRY");
}

#[test]
fn t01_t04_unknown_types_private_never_confirmable_or_public() {
    let (dir,_,draft)=setup();
    let store=ParameterStore::open(dir.path()).unwrap();
    let saved=store.save(proposal(&draft,"restaurant.discount",Some("今天優惠"),0)).unwrap();
    assert!(saved.concept_id.is_none());
    assert_eq!(saved.state,"UNKNOWN");
    assert_eq!(saved.value_type,"unknown");
    assert!(saved.value_json.is_none());
    assert_eq!(saved.raw_value.as_deref(),Some("今天優惠"));
    assert_eq!(saved.registry_version,0);
    assert_eq!(store.confirm(confirmation(&draft,&saved.id,1)).unwrap_err(),
        "PARAMETER_UNAPPROVED_CONCEPT");
    assert!(store.list(&draft).unwrap().iter().all(|p|p.confirmed_event_id.is_none()));
}

#[test]
fn b05_no_goal_no_parameter_for_100_private_screenshots() {
    let dir=tempfile::tempdir().unwrap();
    let assets=LocalStore::open(dir.path()).unwrap();
    for _ in 0..100 {
        assets.save(private_content("パクる")).unwrap();
    }
    let parameters=ParameterStore::open(dir.path()).unwrap();
    let conn=parameters.connect().unwrap();
    let params:i64=conn.query_row("SELECT count(*) FROM local_parameter_drafts",[],
        |r|r.get(0)).unwrap();
    assert_eq!(params,0);
    let uuid=Uuid::new_v4().to_string();
    assert_eq!(parameters.save(proposal(&uuid,"N2",None,0)).unwrap_err(),
        "GOAL_CONFIRMATION_REQUIRED");
}

#[test]
fn b05_non_intent_scope_and_unreviewed_values_rejected() {
    let (dir,_,draft)=setup();
    let store=ParameterStore::open(dir.path()).unwrap();
    for scope in ["profile","pair","global"] {
        let mut input=proposal(&draft,"N2",None,0);
        input.scope=scope.into();
        assert_eq!(store.save(input).unwrap_err(),"INVALID_SCOPE");
    }
    for wrong in ["N1","b2","A1"] {
        let mut input=proposal(&draft,"study.jlpt_level",Some(wrong),0);
        assert_eq!(store.save(input).unwrap_err(),"INVALID_TYPE");
    }
    let mut input=proposal(&draft,"N2",None,0);
    input.expiry_policy="persistent_review".into();
    assert_eq!(store.save(input).unwrap_err(),"INVALID_EXPIRY");
}

#[test]
fn b05_goal_revision_change_invalidates_pending_parameter() {
    let (dir,asset,draft)=setup();
    let store=ParameterStore::open(dir.path()).unwrap();
    let first=store.save(proposal(&draft,"N2",None,0)).unwrap();
    GoalStore::open(dir.path()).unwrap().confirm(ConfirmGoalInput {
        asset_id:asset,goal_text:"真人 N2 學伴，只練聽力".into(),
        confirmation_action:"confirm_goal".into(),expected_revision:1,
    }).unwrap();
    assert!(store.list(&draft).unwrap()[0].stale);
    assert_eq!(store.confirm(confirmation(&draft,&first.id,1)).unwrap_err(),
        "GOAL_REVISION_CONFLICT");
    assert_eq!(store.save(proposal(&draft,"JLPT N2",None,1)).unwrap_err(),
        "GOAL_REVISION_CONFLICT");
    let mut updated=proposal(&draft,"JLPT N2",None,1);
    updated.expected_draft_revision=2;
    let later=store.save(updated).unwrap();
    assert_eq!(later.version,2);
    assert!(!later.stale);
}

#[test]
fn b05_unknown_cannot_forge_event_or_promote_with_sql() {
    let (dir,_,draft)=setup();
    let store=ParameterStore::open(dir.path()).unwrap();
    let saved=store.save(proposal(&draft,"novel_unknown_concept",None,0)).unwrap();
    let conn=store.connect().unwrap();
    assert!(conn.execute(
        "UPDATE local_parameter_drafts
         SET state='CONFIRMED', confirmed_event_id=?1 WHERE id=?2",
        params![Uuid::new_v4().to_string(),saved.id]).is_err());
    assert_eq!(store.list(&draft).unwrap()[0].state,"UNKNOWN");
}

#[test]
fn b05_source_deletion_cascades_parameters_evidence_and_event_history() {
    let (dir,asset,draft)=setup();
    let store=ParameterStore::open(dir.path()).unwrap();
    let saved=store.save(proposal(&draft,"N2",None,0)).unwrap();
    store.confirm(confirmation(&draft,&saved.id,1)).unwrap();
    LocalStore::open(dir.path()).unwrap().delete(&asset).unwrap();
    let conn=store.connect().unwrap();
    for table in ["local_parameter_drafts","local_parameter_evidence",
                  "local_parameter_confirmation_events"] {
        let sql=format!("SELECT count(*) FROM {table}");
        let n:i64=conn.query_row(&sql,[],|r|r.get(0)).unwrap();
        assert_eq!(n,0,"{table} must cascade");
    }
}


#[test]
fn b06_confirmed_known_concept_gets_one_private_slot_and_reopens() {
    let (dir,_,draft)=setup();
    let store=ParameterStore::open(dir.path()).unwrap();
    let proposal=store.save(proposal(&draft,"JLPT N2",None,0)).unwrap();
    assert_eq!(proposal.match_slot_index,None);
    let confirmed=store.confirm(confirmation(&draft,&proposal.id,1)).unwrap();
    assert_eq!(confirmed.state,"CONFIRMED");
    assert_eq!(confirmed.match_slot_index,Some(0));
    assert_eq!(confirmed.visibility,"private");
    assert!(!confirmed.share_consent);
    drop(store);
    let store=ParameterStore::open(dir.path()).unwrap();
    let rows=store.list(&draft).unwrap();
    assert_eq!(rows.len(),1);
    assert_eq!(rows[0].match_slot_index,Some(0));
    let conn=store.connect().unwrap();
    let count:i64=conn.query_row(
        "SELECT COUNT(*) FROM local_matching_slots WHERE draft_id=?1",
        params![draft],|r|r.get(0)).unwrap();
    assert_eq!(count,1);
}

#[test]
fn b06_sqlite_rejects_ninth_slot_direct_tamper_and_unconfirmed_concept() {
    let (dir,_,draft)=setup();
    let store=ParameterStore::open(dir.path()).unwrap();
    let known=store.save(proposal(&draft,"N2",None,0)).unwrap();
    store.confirm(confirmation(&draft,&known.id,1)).unwrap();
    let unknown=store.save(proposal(&draft,"study.unapproved",None,0)).unwrap();
    let conn=store.connect().unwrap();
    assert!(conn.execute(
        "UPDATE local_matching_slots SET slot_index=8 WHERE parameter_id=?1",
        params![known.id]).is_err(),"slots immutable even on direct UPDATE");
    assert!(conn.execute(
        "INSERT INTO local_matching_slots
            (parameter_id,draft_id,concept_id,slot_index,goal_revision,created_at_ms)
         VALUES (?1,?2,'study.jlpt_level',8,1,123)",
        params![unknown.id,draft]).is_err(),"SQLite must reject index 8 / ninth slot");
    assert!(conn.execute(
        "INSERT INTO local_matching_slots
            (parameter_id,draft_id,concept_id,slot_index,goal_revision,created_at_ms)
         VALUES (?1,?2,'study.jlpt_level',1,1,123)",
        params![unknown.id,draft]).is_err(),"Unconfirmed proposal may not forge an active slot");
    let actual:i64=conn.query_row("SELECT count(*) FROM local_matching_slots",[],
        |r|r.get(0)).unwrap();
    assert_eq!(actual,1,"No eviction of existing condition");
}

#[test]
fn b06_edit_source_or_goal_revision_releases_slot_without_auto_approval() {
    let (dir,asset,draft)=setup();
    let store=ParameterStore::open(dir.path()).unwrap();
    let known=store.save(proposal(&draft,"N2",None,0)).unwrap();
    let confirmed=store.confirm(confirmation(&draft,&known.id,1)).unwrap();
    assert_eq!(confirmed.match_slot_index,Some(0));
    let modified=store.save(proposal(&draft,"日語二級",None,2)).unwrap();
    assert_eq!(modified.state,"OBSERVED");
    assert_eq!(modified.match_slot_index,None,"editing requires re-confirmation");
    let confirmed=store.confirm(confirmation(&draft,&modified.id,3)).unwrap();
    assert_eq!(confirmed.match_slot_index,Some(0));
    crate::goal_draft::GoalStore::open(dir.path()).unwrap().confirm(
        ConfirmGoalInput { asset_id:asset,goal_text:"改成只練聽力".into(),
            confirmation_action:"confirm_goal".into(),expected_revision:1}
    ).unwrap();
    let items=store.list(&draft).unwrap();
    assert!(items[0].stale);
    assert_eq!(items[0].match_slot_index,None);
    let conn=store.connect().unwrap();
    let n:i64=conn.query_row("SELECT count(*) FROM local_matching_slots",[],
        |r|r.get(0)).unwrap();
    assert_eq!(n,0);
}

#[test]
fn b06_deleting_original_private_content_cascades_slots() {
    let (dir,asset,draft)=setup();
    let store=ParameterStore::open(dir.path()).unwrap();
    let known=store.save(proposal(&draft,"N2",None,0)).unwrap();
    store.confirm(confirmation(&draft,&known.id,1)).unwrap();
    LocalStore::open(dir.path()).unwrap().delete(&asset).unwrap();
    let conn=store.connect().unwrap();
    let n:i64=conn.query_row("SELECT count(*) FROM local_matching_slots",[],
        |r|r.get(0)).unwrap();
    assert_eq!(n,0);
}

/// T10/T12: exercise exact 8 + ninth boundary with synthetic slot indices.
/// These do NOT authorize eight new concepts in the live T-01 registry.
#[test]
fn b06_eight_slots_full_ninth_rejected_without_eviction() {
    let mut used: Vec<i64> = Vec::new();
    for expected in 0..8_i64 {
        let slot = lowest_free_matching_slot(&used).unwrap();
        assert_eq!(slot, expected);
        used.push(slot);
    }
    assert_eq!(lowest_free_matching_slot(&used).unwrap_err(),
        "B06_SLOT_BUDGET_EXCEEDED");
    assert_eq!(used, (0_i64..8).collect::<Vec<_>>(),
        "the ninth must never evict an existing hard constraint");
    used.remove(3);
    assert_eq!(lowest_free_matching_slot(&used).unwrap(), 3,
        "a genuinely released slot is reused deterministically");
}
