# Input

`````vue
<script setup lang="tsx">
const short = (
  <div>
    <label for="d" class={[c]}>
      Please enter the scheduled payment date below
    </label>
  </div>
);
const long = <div>Please enter the scheduled payment date below aaa bbb ccc ddd eee fff ggg hhh</div>;
</script>

`````

# Output

## `{"printWidth":80}`

`````vue
<script setup lang="tsx">
const short = (
  <div>
    <label for="d" class={[c]}>
      Please enter the scheduled payment date below
    </label>
  </div>
);
const long = (
  <div>
    Please enter the scheduled payment date below aaa bbb ccc ddd eee fff ggg
    hhh
  </div>
);
</script>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<script setup lang="tsx">
  const short = (
    <div>
      <label for="d" class={[c]}>
        Please enter the scheduled payment date below
      </label>
    </div>
  )
  const long = (
    <div>Please enter the scheduled payment date below aaa bbb ccc ddd eee fff ggg hhh</div>
  )
</script>

`````
