# Input

`````js
import styled from 'styled-components';

const Example = styled.div`
  content: '\\201C';
`;

export default Example;

`````

# Output

## `{"printWidth":80}`

`````js
import styled from "styled-components";

const Example = styled.div`
  content: "\\201C";
`;

export default Example;

`````

## `{"printWidth":100}`

`````js
import styled from "styled-components";

const Example = styled.div`
  content: "\\201C";
`;

export default Example;

`````
