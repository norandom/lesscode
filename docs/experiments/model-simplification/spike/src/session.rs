//! Foreground prototype bridge: the existing repair loop awaits the frozen step verdict.
use lesscode_model::{CallError, Reply};
use std::{fs, io, path::Path};

pub fn drive(args: &[String]) -> Result<(), String> {
    if args.len() != 4 {
        return Err("drive REQUEST OUTPUT live|stub ENDPOINT_OR_REPLIES".into());
    }
    let output = Path::new(&args[1]);
    validate_output(output)?;
    let request = lesscode_model::protocol::read_request(
        &fs::read_to_string(&args[0]).map_err(|e| e.to_string())?,
    )?;
    if request.max_output_tokens != 8192 {
        return Err("output ceiling must be 8192".into());
    }
    let endpoint = match args[2].as_str() {
        "live" => Some(
            lesscode_model::endpoint_from(Some(Path::new(&args[3]))).map_err(|e| e.to_string())?,
        ),
        "stub" => None,
        _ => return Err("mode must be live or stub".into()),
    };
    let mut stubs = if endpoint.is_none() {
        serde_json::from_str::<Vec<Reply>>(
            &fs::read_to_string(&args[3]).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?
    } else {
        Vec::new()
    }
    .into_iter();
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    let mut number = 0;
    let record = super::lifecycle::trial(
        &mut |asked| {
            number += 1;
            write(&output.join(format!("request-{number}.json")), asked)
                .map_err(CallError::Config)?;
            let answer = if let Some(endpoint) = &endpoint {
                lesscode_model::call(
                    endpoint,
                    asked,
                    &lesscode_model::Limits::default(),
                    &lesscode_model::Terminal,
                    Some(&output.join("calls.jsonl")),
                )
            } else {
                stubs
                    .next()
                    .ok_or_else(|| CallError::Config("stub exhausted".into()))
            };
            write(
                &output.join(format!("reply-{number}.json")),
                &lesscode_model::protocol::answer(&answer),
            )
            .map_err(CallError::Config)?;
            answer
        },
        &request,
        &mut |text| judge(output, text),
    );
    let result = match &record.outcome {
        lesscode_model::repair::Outcome::Accepted { .. } => "accepted",
        lesscode_model::repair::Outcome::Rejected { .. } => "refused",
        lesscode_model::repair::Outcome::Failed { .. } => "incomplete",
    };
    write(
        &output.join("trial.json"),
        &serde_json::json!({"result": result,
        "judge_failure": record.judge_failure, "requests": record.requests,
        "replies": record.replies, "intervention": "foreground controller supplies frozen step verdicts"}),
    )?;
    println!("{result}");
    Ok(())
}

fn write(path: &Path, value: &impl serde::Serialize) -> Result<(), String> {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn validate_output(output: &Path) -> Result<(), String> {
    let absolute = output.to_string_lossy().replace('\\', "/");
    if !absolute.starts_with("D:/BuildArtifacts/lesscode/model-simplification/ms1/")
        || absolute.split('/').any(|p| p == "..")
        || output.exists()
    {
        return Err("output must be a new directory inside the experiment root".into());
    }
    let root = fs::canonicalize("D:/BuildArtifacts/lesscode/model-simplification/ms1")
        .map_err(|e| e.to_string())?;
    let parent = fs::canonicalize(output.parent().ok_or("output has no parent")?)
        .map_err(|e| e.to_string())?;
    if !parent.starts_with(root) || fs::symlink_metadata(output).is_ok() {
        return Err("output escapes resolved experiment root or already exists".into());
    }
    Ok(())
}

fn judge(output: &Path, text: &str) -> Result<lesscode_model::repair::Verdict, String> {
    // The foreground controller records scope/hash checks and the real step verdict
    // before replying. No candidate path or executable is taken from model output.
    fs::write(output.join("pending-source.py"), text).map_err(|e| e.to_string())?;
    println!("JUDGE {}", output.join("pending-source.py").display());
    let mut line = String::new();
    if io::stdin()
        .read_line(&mut line)
        .map_err(|e| e.to_string())?
        == 0
    {
        return Err("judge input closed".into());
    }
    let value: serde_json::Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
    let reasons = value["reasons"]
        .as_array()
        .ok_or("judge reasons missing")?
        .iter()
        .map(|v| {
            v.as_str()
                .map(String::from)
                .ok_or("non-string reason".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    match value["result"].as_str() {
        Some("accepted") => Ok(lesscode_model::repair::Verdict {
            passed: true,
            reasons,
        }),
        Some("refused") => Ok(lesscode_model::repair::Verdict {
            passed: false,
            reasons,
        }),
        _ => Err(format!("judge incomplete: {}", reasons.join("; "))),
    }
}
