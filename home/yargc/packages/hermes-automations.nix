{
  git,
  hermesAgent,
  libnotify,
  python3,
  systemd,
  writeShellApplication,
}:
writeShellApplication {
  name = "vesper-hermes-automations";

  runtimeInputs = [
    git
    hermesAgent
    libnotify
    python3
    systemd
  ];

  text = ''
    export VESPER_HERMES_AUTOMATION_BIN="$0"
    export PYTHONPATH="${./.}:''${PYTHONPATH:-}"
    exec python3 ${./hermes-automations.py} "$@"
  '';
}
