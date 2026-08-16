{ inputs, pkgs }:
let
  agents = inputs.llm-agents.packages.${pkgs.system};
in
agents.hermes-agent.override (old: {
  python3 = old.python3.override {
    packageOverrides = _final: prev: {
      slack-bolt = prev.slack-bolt.overridePythonAttrs (_: {
        doCheck = false;
      });
    };
  };
})
