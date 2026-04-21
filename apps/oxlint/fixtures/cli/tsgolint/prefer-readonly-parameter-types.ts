interface MutableInput {
  value: string;
}

function consume(input: MutableInput): void {
  input.value = input.value.trim();
}
