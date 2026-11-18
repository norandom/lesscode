    // --- R154 experiment harness -------------------------------------------
    // Inserted before the last line of `tools/guide-optimizer/src/adapter.rs`,
    // inside `mod trials`, in a copy of the tree. It runs the real `one` and
    // the real `with_repairs` on scripted reply histories. A reply code is
    // 0 complete with an admissible pack, 1 complete with `{}`, 2 cut off
    // with the pack, 3 refused with the pack, 4 a failed call.

    struct Script(std::sync::Mutex<std::collections::VecDeque<u64>>);

    impl Script {
        fn of(case: &Value) -> Script {
            let codes = case["replies"].as_array().expect("replies");
            Script(std::sync::Mutex::new(
                codes.iter().map(|c| c.as_u64().expect("code")).collect(),
            ))
        }
    }

    impl Model for Script {
        fn name(&self) -> String {
            "scripted".into()
        }

        fn chat(&self, request: &CallRequest) -> Result<Reply, CallError> {
            let code = self.0.lock().unwrap().pop_front().expect("a reply per call");
            let stop = match code {
                4 => return Err(CallError::Config("scripted failure".into())),
                2 => Stop::CutOff,
                3 => Stop::Refused,
                _ => Stop::Complete,
            };
            let mut reply = Ending(stop).chat(request)?;
            if code == 1 {
                reply.text = "{}".into();
            }
            Ok(reply)
        }
    }

    /// What a check is handed: the text before the consolidation, the reply after.
    trait Seen {
        fn text(&self) -> &str;
    }

    impl Seen for str {
        fn text(&self) -> &str {
            self
        }
    }

    impl Seen for Reply {
        fn text(&self) -> &str {
            &self.text
        }
    }

    const REPAIRS: usize = 2;

    fn through_the_adapter(case: &Value) -> Value {
        let ledger = std::env::temp_dir().join(format!("r154-{}.jsonl", std::process::id()));
        let model = Metered::new(
            Box::new(Script::of(case)),
            (0.0, 0.0),
            1.0,
            f64::INFINITY,
            ledger.clone(),
        );
        let java = Language {
            name: "java".into(),
            request: "Write the pack for: java".into(),
            kinds: String::new(),
            scopes: Vec::new(),
        };
        let mut authoring = Authoring::new(
            Arc::new(model),
            Arc::new(BTreeMap::from([("java".to_owned(), java)])),
            Vec::new(),
            Vec::new(),
            Arc::new(Rules::new("", Vec::new())),
        );
        authoring.repairs = REPAIRS;
        let record = authoring.one("final", "guide", "java").record;
        let _ = std::fs::remove_file(ledger);
        record
    }

    /// The loop with a check that judges the text alone, as the callers in
    /// `lesscode-model` do.
    fn through_a_plain_check(case: &Value) -> Value {
        let script = Script::of(case);
        let request = CallRequest {
            purpose: "r154".into(),
            system: String::new(),
            user: "Write the pack for: java".into(),
            max_output_tokens: 100,
            temperature: None,
            structure: None,
        };
        let outcome = with_repairs(
            &mut |asked| script.chat(asked),
            &request,
            &mut |seen| repair::Verdict {
                passed: Seen::text(seen) != "{}",
                reasons: vec!["the pack is empty".into()],
            },
            REPAIRS,
        );
        let (word, attempts) = match outcome {
            Outcome::Accepted { attempts, .. } => ("accepted", attempts),
            Outcome::Rejected { attempts, .. } => ("rejected", attempts),
            Outcome::Failed { attempts, .. } => ("failed", attempts),
        };
        let attempts: Vec<Value> = attempts
            .iter()
            .map(|a| json!([a.passed, stop_word(&a.stop)]))
            .collect();
        json!({"outcome": word, "attempts": attempts})
    }

    fn histories(run: fn(&Value) -> Value) {
        let path = |name: &str| std::env::var(name).unwrap_or_else(|_| panic!("{name} is not set"));
        let cases: Vec<Value> =
            serde_json::from_slice(&std::fs::read(path("HISTORY_IN")).unwrap()).unwrap();
        let reports: Vec<Value> = cases
            .iter()
            .map(|case| json!({"id": case["id"], "value": run(case)}))
            .collect();
        std::fs::write(path("HISTORY_OUT"), json!({"reports": reports}).to_string()).unwrap();
    }

    #[test]
    #[ignore = "experiment harness: needs HISTORY_IN and HISTORY_OUT"]
    fn histories_through_the_adapter() {
        histories(through_the_adapter);
    }

    #[test]
    #[ignore = "experiment harness: needs HISTORY_IN and HISTORY_OUT"]
    fn histories_through_a_plain_check() {
        histories(through_a_plain_check);
    }
