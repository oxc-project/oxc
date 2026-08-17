
    /**
     * Utility type for getting the values for specific style keys.
     * # test:
     * The following is bad because position is more restrictive than 'string':
     * ```
     * type Props = {position: string};
     * ```
     *
     * You should use the following instead:
     *
     * ```
     * type Props = {position: TypeForStyleKey<'position'>};
     * ```
     *
     * This will correctly give you the type 'absolute' | 'relative'
     */
