"use client";

import { Fragment, memo, useMemo, useState } from "react";
import { rotateArray } from "@giraugh/tools";
import { Temporal } from "@js-temporal/polyfill";
import { ChevronLeft, ChevronRight } from "lucide-react";

import Button from "/src/components/Button/Button";
import { useTranslation } from "/src/i18n/client";
import { useStore } from "/src/stores";
import useSettingsStore from "/src/stores/settingsStore";
import { getWeekdayNames } from "/src/utils";

import styles from "/src/components/CalendarField/components/Month/Month.module.scss";

export interface DayCell {
  date: Temporal.PlainDate;
  /** PlainDate ISO string `YYYY-MM-DD` */
  iso: string;
  isOtherMonth: boolean;
  isToday: boolean;
  /** True if this date is one of the event's days */
  isInEvent: boolean;
  x: number;
  y: number;
}

interface DayGridProps {
  /** Set of `YYYY-MM-DD` strings — the days the event covers */
  eventDates: Set<string>;
  renderCell: (cell: DayCell) => React.ReactNode;
}

const DayGrid = ({ eventDates, renderCell }: DayGridProps) => {
  const { t, i18n } = useTranslation("home");

  const weekStart = useStore(useSettingsStore, (state) => state.weekStart) ?? 0;

  // Default the page to the first event date (or today).
  const initialPage = useMemo(() => {
    const first = [...eventDates].sort()[0];
    return first
      ? Temporal.PlainDate.from(first).toPlainYearMonth()
      : Temporal.Now.plainDateISO().toPlainYearMonth();
  }, [eventDates]);

  const [page, setPage] = useState<Temporal.PlainYearMonth>(initialPage);

  const grid = useMemo(() => calculateMonth(page, weekStart), [page, weekStart]);

  return (
    <>
      <div className={styles.header}>
        <Button
          title={t("form.dates.tooltips.previous")}
          onClick={() => setPage(page.subtract({ months: 1 }))}
          icon={<ChevronLeft />}
        />
        <span>
          {page
            .toPlainDate({ day: 1 })
            .toLocaleString(i18n.language, { month: "long", year: "numeric" })}
        </span>
        <Button
          title={t("form.dates.tooltips.next")}
          onClick={() => setPage(page.add({ months: 1 }))}
          icon={<ChevronRight />}
        />
      </div>

      <div className={styles.dayLabels}>
        {rotateArray(getWeekdayNames(i18n.language, "short"), weekStart ? 0 : 1).map((name) => (
          <label key={name}>{name}</label>
        ))}
      </div>

      <div className={styles.grid}>
        {grid.map((row, y) =>
          row.map((date, x) => {
            const iso = date.toString();
            return (
              <Fragment key={iso}>
                {renderCell({
                  date,
                  iso,
                  isOtherMonth: date.month !== page.month,
                  isToday: date.equals(Temporal.Now.plainDateISO()),
                  isInEvent: eventDates.has(iso),
                  x,
                  y,
                })}
              </Fragment>
            );
          }),
        )}
      </div>
    </>
  );
};

const calculateMonth = (
  month: Temporal.PlainYearMonth,
  weekStart: 0 | 1,
): Temporal.PlainDate[][] => {
  const daysBefore = month.toPlainDate({ day: 1 }).dayOfWeek - weekStart;
  const daysAfter = 6 - month.toPlainDate({ day: month.daysInMonth }).dayOfWeek + weekStart;

  const dates: Temporal.PlainDate[][] = [];
  let curDate = month.toPlainDate({ day: 1 }).subtract({ days: daysBefore });
  let y = 0;
  let x = 0;
  for (let i = 0; i < daysBefore + month.daysInMonth + daysAfter; i++) {
    if (x === 0) dates[y] = [];
    dates[y][x] = curDate;
    curDate = curDate.add({ days: 1 });
    x++;
    if (x > 6) {
      x = 0;
      y++;
    }
  }
  return dates;
};

export default memo(DayGrid);
