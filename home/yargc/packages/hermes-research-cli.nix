{
  curl,
  hermesAgent,
  python3,
  writeShellApplication,
}:
writeShellApplication {
  name = "vesper-research";

  runtimeInputs = [
    curl
    hermesAgent
    python3
  ];

  text = ''
    export PYTHONPATH="${./.}:''${PYTHONPATH:-}"
    exec python3 ${./hermes-research-cli.py} "$@"
  '';
}
