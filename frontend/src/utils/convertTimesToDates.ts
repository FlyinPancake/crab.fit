import { Temporal } from "@js-temporal/polyfill";

/**
 * Take times as strings in UTC and convert to ZonedDateTime objects in the timezone supplied
 * @param times An array of strings in `HHmm-d` or `HHmm-DDMMYYYY` format
 * @param timezone The target timezone
 */
export const convertTimesToDates = (
  times: string[],
  timezone: string,
): Temporal.ZonedDateTime[] => {
  const isSpecificDates = times[0]?.length === 13;

  return times.map((time) =>
    isSpecificDates
      ? parseSpecificDate(time).withTimeZone(timezone)
      : parseWeekdayDate(time, timezone).withTimeZone(timezone),
  );
};

/**
 * Day-based event detection. Day-based time strings have length 1 (`d` weekday)
 * or 8 (`DDMMYYYY` specific date). Time-based strings are 6 or 13 chars.
 */
export const isDayBased = (times: string[]): boolean => {
  const len = times[0]?.length;
  return len === 1 || len === 8;
};

export const isDaySpecific = (times: string[]): boolean => times[0]?.length === 8;

/** Parse a `DDMMYYYY` string into a PlainDate. */
export const parseSpecificDay = (str: string): Temporal.PlainDate => {
  if (str.length !== 8) {
    throw new Error("String must be in DDMMYYYY format");
  }
  const day = Number(str.substring(0, 2));
  const month = Number(str.substring(2, 4));
  const year = Number(str.substring(4));
  return Temporal.PlainDate.from({ year, month, day });
};

/** Parse a single-digit weekday string (0=Sun, 1=Mon, ..., 6=Sat). */
export const parseWeekdayDay = (str: string): number => {
  if (str.length !== 1) {
    throw new Error("String must be in d format");
  }
  return Number(str);
};

// Parse from UTC `HHmm-DDMMYYYY` format into a ZonedDateTime in UTC
export const parseSpecificDate = (str: string): Temporal.ZonedDateTime => {
  if (str.length !== 13) {
    throw new Error("String must be in HHmm-DDMMYYYY format");
  }

  // Extract values
  const [hour, minute] = [Number(str.substring(0, 2)), Number(str.substring(2, 4))];
  const [day, month, year] = [
    Number(str.substring(5, 7)),
    Number(str.substring(7, 9)),
    Number(str.substring(9)),
  ];

  // Construct PlainDateTime
  return Temporal.ZonedDateTime.from({
    hour,
    minute,
    day,
    month,
    year,
    timeZone: "UTC",
  });
};

// Parse from UTC `HHmm-d` format into a ZonedDateTime in UTC based on the current date
const parseWeekdayDate = (str: string, timezone: string): Temporal.ZonedDateTime => {
  if (str.length !== 6) {
    throw new Error("String must be in HHmm-d format");
  }

  // Extract values
  const [hour, minute] = [Number(str.substring(0, 2)), Number(str.substring(2, 4))];
  const dayOfWeek = Number(str.substring(5));

  // Construct PlainDateTime from today
  const today = Temporal.Now.zonedDateTimeISO("UTC").round("day");
  const dayDelta = dayOfWeek - today.dayOfWeek;
  const resultDay = today.add({ days: dayDelta });

  let resultDate = resultDay.with({
    hour,
    minute,
  });

  // If resulting day (in target timezone) is in the next week, move it back to this week
  // TODO: change data representation instead
  const dayInTz = resultDate.withTimeZone(timezone);
  const todayInTz = today.withTimeZone(timezone);
  if (dayInTz.weekOfYear && todayInTz.weekOfYear && dayInTz.weekOfYear > todayInTz.weekOfYear) {
    resultDate = resultDate.subtract({ days: 7 });
  }

  return resultDate;
};
