{
  writeShellApplication,
  postgresql,
  ...
}:
writeShellApplication {
  name = "dev-pg";
  runtimeInputs = [
    postgresql
  ];
  text = builtins.readFile ./dev-pg.sh;
}