import styled from 'styled-components';

// Test case from issue #13312
const StyledRemoveButton = styled.button``;
const SidebarDragHandle = styled.span``;
const RightSideWrapper = styled.div``;

const Wrapper = styled("div").withConfig({
  displayName: "main__Wrapper",
})`
  position: relative;
  display: block;
  ${StyledRemoveButton} {
    opacity: 0;
  }
  border: 1px black solid;
  @media (hover: hover) {
    &:hover ${SidebarDragHandle} {
      display: inline-block;
    }
    &:hover ${RightSideWrapper} ${StyledRemoveButton} {
      opacity: 0;
    }
    &:hover
      ${StyledRemoveButton},&
      ${RightSideWrapper}:hover
      ${StyledRemoveButton} {
      opacity: 1;
    }
  }
`;

// Additional test cases for consecutive interpolations
const TestConsecutive = styled.div`
  ${StyledRemoveButton} ${SidebarDragHandle} {
    color: red;
  }
  .class ${StyledRemoveButton} ${SidebarDragHandle} ${RightSideWrapper} {
    color: blue;
  }
`;
