# Input

`````vue
<script lang="tsx">
export default {
  render( h ): VNode {return <div>{ this.foo   }</div>    },
}
</script>

`````

# Output

## `{"printWidth":80}`

`````vue
<script lang="tsx">
export default {
  render(h): VNode {
    return <div>{this.foo}</div>;
  },
};
</script>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<script lang="tsx">
  export default {
    render(h): VNode {
      return <div>{this.foo}</div>
    },
  }
</script>

`````
