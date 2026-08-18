{
  callPackage,
  coreutils,
  curl,
  ghostty,
  git,
  hermesAgent,
  jq,
  lib,
  libnotify,
  makeWrapper,
  rustc,
  stdenv,
  systemd,
  writeShellApplication,
}:
let
  vesperControl = callPackage ./vesper-control.nix { };

  hermesCredentialProxy = writeShellApplication {
    name = "hermes";
    runtimeInputs = [
      coreutils
      vesperControl
    ];
    text = ''
      credential="$(${vesperControl}/bin/vesper-control consumer credential hermes)"
      exec ${vesperControl}/bin/vesper-control credential exec "$credential" -- \
        ${hermesAgent}/bin/hermes "$@"
    '';
  };

  hermesJobsStatus = writeShellApplication {
    name = "vesper-hermes-jobs-status";
    runtimeInputs = [ coreutils jq ];
    text = ''
      registry="''${VESPER_HERMES_JOB_REGISTRY:-$HOME/.config/vesper/hermes-jobs.json}"
      state="''${VESPER_RESEARCH_STATE_DIR:-$HOME/.local/state/vesper/research}"

      if [ ! -f "$registry" ]; then
        printf '{}\n'
        exit 0
      fi

      {
        ${jq}/bin/jq -c 'to_entries[]' "$registry" | while IFS= read -r entry; do
          task="$(printf '%s\n' "$entry" | ${jq}/bin/jq -r '.value.task // .key')"
          last='{}'
          if [[ "$task" =~ ^[A-Za-z0-9._-]+$ ]]; then
            latest="$state/runs/$task/latest.json"
            if [ -f "$latest" ]; then
              last="$(${jq}/bin/jq -c 'if type == "object" then . else {} end' "$latest" 2>/dev/null || printf '{}')"
            fi
          fi
          printf '%s\n' "$entry" | ${jq}/bin/jq -c --argjson last "$last" '.value.lastRun = $last'
        done
      } | ${jq}/bin/jq -s 'from_entries'
    '';
  };
in
stdenv.mkDerivation {
  pname = "vesper-hermes-core";
  version = "2.1.0";

  src = ./hermes-rs;

  nativeBuildInputs = [
    makeWrapper
    rustc
  ];

  dontConfigure = true;

  buildPhase = ''
    runHook preBuild
    rustc --edition=2021 -C opt-level=2 main.rs -o vesper-hermes-core
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    install -Dm755 vesper-hermes-core $out/bin/vesper-hermes-core
    wrapProgram $out/bin/vesper-hermes-core \
      --set-default HERMES_RESEARCH_PROVIDER xai \
      --prefix PATH : ${lib.makeBinPath [
        coreutils
        curl
        ghostty
        git
        hermesCredentialProxy
        jq
        libnotify
        systemd
      ]}

    ln -s vesper-hermes-core $out/bin/vesper-hermes
    ln -s vesper-hermes-core $out/bin/vesper-hermes-automations
    ln -s vesper-hermes-core $out/bin/vesper-research
    ln -s ${hermesJobsStatus}/bin/vesper-hermes-jobs-status $out/bin/vesper-hermes-jobs-status

    runHook postInstall
  '';

  meta = {
    description = "Native Rust control plane for Vesper Hermes research and automation";
    license = lib.licenses.mit;
    mainProgram = "vesper-hermes";
    platforms = lib.platforms.linux;
  };
}
