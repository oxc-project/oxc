// https://github.com/rolldown/rolldown/issues/8866
const enum Theme {
  Light = "Light",
  Dark = "Dark",
  Default = Theme.Light,
}
console.log(Theme.Light);
console.log(Theme.Default);

enum Color {
  Red = "Red",
  Green = "Green",
  Primary = Color.Red,
}
Color.Red;
Color.Primary;
