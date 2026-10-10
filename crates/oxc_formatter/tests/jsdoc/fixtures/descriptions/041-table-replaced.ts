
    /**
     * @param options Options object for setting file system flags (default: `"r"`):
     *
     * | flags   | description                                                                                                    |
     * | ------- | -------------------------------------------------------------------------------------------------------------- |
     * | `"a"`   | Open file for appending. The file is created if it does not exist.                                             |
     * | `"ax"`  | Like `"a"` but fails if the path exists.                                                                       |
     * | `"a+"`  | Open file for reading and appending. The file is created if it does not exist.                                 |
     * | `"ax+"` | Like `"a+"` but fails if the path exists.                                                                      |
     * | `"as"`  | Open file for appending in synchronous mode. The file is created if it does not exist.                         |
     * | `"as+"` | Open file for reading and appending in synchronous mode. The file is created if it does not exist.             |
     * | `"r"`   | Open file for reading. An exception occurs if the file does not exist.                                         |
     * | `"r+"`  | Open file for reading and writing. An exception occurs if the file does not exist.                             |
     * | `"rs+"` | Open file for reading and writing in synchronous mode. Instructs the OS to bypass the local file system cache. |
     * | `"w"`   | Open file for writing. The file is created (if it does not exist) or truncated (if it exists).                 |
     * | `"wx"`  | Like `"w"` but fails if the path exists.                                                                       |
     * | `"w+"`  | Open file for reading and writing. The file is created (if it does not exist) or truncated (if it exists).     |
     * | `"wx+"` | Like `"w+"` but fails if the path exists.                                                                      |
     */
