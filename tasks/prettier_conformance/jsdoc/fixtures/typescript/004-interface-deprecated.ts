
export interface FetchCallbackResponseArray<T, V> {
  resource: Resource<T>;
      /**
       * @deprecated Resolve clear with condition in your fetch api this function will be remove
       */
  refetch: (...arg: V[]) => void;
  /**
   * @deprecated Resolve clear with condition in your fetch api this function will be remove
   */
  clear: () => void;
}
