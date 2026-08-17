const ConfirmationCodeInput = React.lazy(
  () => import('ConfirmationCodeInput.react')
) as React.ComponentType<React.ElementConfig<ConfirmationCodeInputType>>;
export const examples = [
  {
    render(): React.MixedElement {
      return (
        <ConfirmationCodeInput
          helperText={isCompleted && `Your code entry is completed: ${value}`}
          label={fbt()}
        />
      );
    },
  },
];
