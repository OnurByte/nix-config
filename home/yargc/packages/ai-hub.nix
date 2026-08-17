{
  agentCockpit,
  codexbar,
  hermesRuntime,
  python3,
  writeShellApplication,
}:
writeShellApplication {
  name = "vesper-ai-hub";

  runtimeInputs = [
    python3
    codexbar
    agentCockpit
    hermesRuntime
  ];

  text = ''
    exec python3 ${./ai-hub.py} "$@"
  '';
}
