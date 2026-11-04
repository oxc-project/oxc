<script>
export default {
  emits: ["notify"],
};
debugger;
</script>

<script setup>
// `emits` is declared in the other `<script>` block too, so this is an error.
defineEmits(["submit"]);
debugger;
</script>
