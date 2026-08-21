
  /**
   * Bounce give a renderContent and show that around children when isVisible is
   * true
   *
   * @satisfies {React.FC<BounceProps>}
   * @example
   *   <Bounce
   *     isVisible={isVisible}
   *     dismiss={() => setVisible(false)}
   *     renderContent={() => {
   *       return <InsideOfPopeUp />;
   *     }}>
   *     <Button />
   *   </Bounce>;
   *
   * @type {React.FC<BounceProps>}
   */
