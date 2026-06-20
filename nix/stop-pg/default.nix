{
  writeShellApplication,
  postgresql,
  ...
}:
writeShellApplication {
  name = "stop-pg";
  runtimeInputs = [
    postgresql
  ];
  text = builtins.readFile ./stop-pg.sh;
}
