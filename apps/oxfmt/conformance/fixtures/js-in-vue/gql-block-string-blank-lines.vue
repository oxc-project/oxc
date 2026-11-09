<script setup>
const q = graphql`
  """
  First paragraph.


  Second paragraph after two blanks.
  """
  type Query {
    hello: String
  }
`;
</script>
