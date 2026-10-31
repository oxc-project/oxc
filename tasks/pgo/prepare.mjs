import { workerData } from "node:worker_threads";
import { prepareCorpus } from "./corpus.mjs";

await prepareCorpus(workerData.directory, workerData.ecosystemRoot, workerData.fetch);
