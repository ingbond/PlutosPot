import winston, { Logger } from "winston";

export let logger: Logger;

export function initLogger() {
  logger = winston.createLogger({
    level: "info",
    format: winston.format.json(),
    defaultMeta: { service: "user-service" },
    transports: [
      //
      // - Write all logs with level `error` and below to `error.log`
      // - Write all logs with level `info` and below to `combined.log`
      //
      new winston.transports.File({ filename: "error.log", level: "error" }),
      new winston.transports.File({ filename: "combined.log" }),
    ],
  });

  if (process.env.MODE == "debug") {
    logger.add(
      new winston.transports.Console({
        format: winston.format.simple(),
      })
    );
  }
}
