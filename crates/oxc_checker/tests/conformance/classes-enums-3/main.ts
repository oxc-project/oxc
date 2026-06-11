import { Animal, Cat, Dog, Mood, Speaker } from "./animals";

const ok_dog: Dog = new Dog("rex");
const ok_animal: Animal = new Cat("tom");
const ok_speaker: Speaker = ok_dog;
const ok_line: string = ok_animal.speak();
const ok_lives: number = new Cat("mia").lives;

const ok_mood: Mood = Mood.Happy;
const ok_mood_as_string: string = ok_mood;

function pick_mood(is_happy: boolean): Mood {
  return is_happy ? Mood.Happy : Mood.Grumpy;
}

const ok_picked: Mood = pick_mood(true);
const ok_grumpy_member: Mood.Grumpy = Mood.Grumpy;
