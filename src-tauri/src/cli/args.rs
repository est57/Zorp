/// Options of `zorp run`.
#[derive(Debug, Default, PartialEq)]
pub struct RunArgs {
    pub folder: String,
    pub collection: Option<String>,
    pub folder_name: Option<String>,
    pub environment: Option<String>,
    pub data: Option<String>,
    pub vars: Vec<(String, String)>,
    pub delay_ms: u32,
    pub bail: bool,
    pub reporter: Reporter,
    pub out: Option<String>,
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum Reporter {
    #[default]
    Text,
    Json,
    Junit,
}

pub const USAGE: &str = "Pemakaian: zorp-cli run <folder-workspace> [opsi]
          (dari aplikasi terpasang: zorp run <folder-workspace> [opsi])

Opsi:
  --collection <nama>   Koleksi yang dijalankan (wajib jika ada lebih dari satu)
  --folder <nama>       Hanya folder ini di dalam koleksi
  --env <nama>          Environment yang dipakai
  --data <file>         File CSV atau JSON, satu iterasi per baris
  --env-var <k=v>       Isi atau timpa variabel; boleh diulang. ZORP_VAR_<nama> juga dibaca
  --delay <ms>          Jeda antar request
  --bail                Berhenti pada kegagalan pertama
  --reporter <jenis>    text (bawaan), json, atau junit
  --out <file>          Tulis laporan json atau junit ke file";

fn value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next().ok_or_else(|| format!("{flag} butuh nilai"))
}

/// Parses the arguments after `run`.
pub fn parse(mut args: impl Iterator<Item = String>) -> Result<RunArgs, String> {
    let mut out = RunArgs::default();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--collection" => out.collection = Some(value(&mut args, &arg)?),
            "--folder" => out.folder_name = Some(value(&mut args, &arg)?),
            "--env" => out.environment = Some(value(&mut args, &arg)?),
            "--data" => out.data = Some(value(&mut args, &arg)?),
            "--env-var" => {
                let pair = value(&mut args, &arg)?;
                let (k, v) = pair
                    .split_once('=')
                    .ok_or("--env-var memakai format nama=nilai")?;
                out.vars.push((k.trim().to_string(), v.to_string()));
            }
            "--delay" => {
                out.delay_ms = value(&mut args, &arg)?
                    .parse()
                    .map_err(|_| "--delay harus angka")?
            }
            "--bail" => out.bail = true,
            "--reporter" => {
                out.reporter = match value(&mut args, &arg)?.as_str() {
                    "text" => Reporter::Text,
                    "json" => Reporter::Json,
                    "junit" => Reporter::Junit,
                    other => return Err(format!("Reporter {other} tidak dikenal")),
                }
            }
            "--out" => out.out = Some(value(&mut args, &arg)?),
            "-h" | "--help" => return Err(USAGE.into()),
            flag if flag.starts_with("--") => {
                return Err(format!("Opsi {flag} tidak dikenal\n\n{USAGE}"));
            }
            _ if out.folder.is_empty() => out.folder = arg,
            extra => return Err(format!("Argumen tidak terduga: {extra}")),
        }
    }
    if out.folder.is_empty() {
        return Err(USAGE.into());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_run_options() {
        let args = [
            "./ws",
            "--collection",
            "API",
            "--env",
            "Staging",
            "--env-var",
            "token=a=b",
            "--bail",
            "--reporter",
            "junit",
        ];
        let parsed = parse(args.into_iter().map(String::from)).unwrap();
        assert_eq!(parsed.folder, "./ws");
        assert_eq!(parsed.vars, [("token".to_string(), "a=b".to_string())]);
        assert!(parsed.bail);
        assert_eq!(parsed.reporter, Reporter::Junit);
        assert!(parse(["--env"].into_iter().map(String::from)).is_err());
        assert!(parse(["./ws", "--nope"].into_iter().map(String::from)).is_err());
    }
}
