"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { flip, offset, shift, useFloating } from "@floating-ui/react-dom";
import { Temporal } from "@js-temporal/polyfill";

import Content from "/src/components/Content/Content";
import DayGrid, { DayCell } from "/src/components/DayGrid/DayGrid";
import Legend from "/src/components/Legend/Legend";
import { PersonResponse } from "/src/app/actions";
import { usePalette } from "/src/hooks/usePalette";
import { useTranslation } from "/src/i18n/client";
import { useStore } from "/src/stores";
import useSettingsStore from "/src/stores/settingsStore";
import {
  calculateAvailability,
  makeClass,
  parseSpecificDay,
  relativeTimeFormat,
} from "/src/utils";

import styles from "./AvailabilityViewer.module.scss";
import monthStyles from "/src/components/CalendarField/components/Month/Month.module.scss";

interface AvailabilityViewerDaysProps {
  /** Event days as `DDMMYYYY` strings */
  times: string[];
  people: PersonResponse[];
}

const AvailabilityViewerDays = ({ times, people }: AvailabilityViewerDaysProps) => {
  const { t, i18n } = useTranslation("event");

  const highlight = useStore(useSettingsStore, (state) => state.highlight);
  const [filteredPeople, setFilteredPeople] = useState(people.map((p) => p.name));
  const [tempFocus, setTempFocus] = useState<string>();
  const [focusCount, setFocusCount] = useState<number>();

  const [tooltip, setTooltip] = useState<{
    anchor: HTMLElement;
    available: string;
    date: string;
    people: string[];
  }>();
  const { refs, floatingStyles } = useFloating({
    middleware: [offset(6), flip(), shift()],
    elements: { reference: tooltip?.anchor },
  });

  // Calculate availability per day (uses serialized DDMMYYYY strings)
  const { availabilities, min, max } = useMemo(
    () =>
      calculateAvailability(
        times,
        people.filter((p) => filteredPeople.includes(p.name)),
      ),
    [times, filteredPeople, people],
  );

  const palette = usePalette(Math.max(max - min + 1, 2));

  useEffect(() => {
    setFilteredPeople(people.map((p) => p.name));
  }, [people.length]);

  const eventIsoSet = useMemo(
    () => new Set(times.map((s) => parseSpecificDay(s).toString())),
    [times],
  );

  // Map ISO `YYYY-MM-DD` back to the stored `DDMMYYYY` for availability lookup
  const isoToSerialized = useMemo(() => {
    const map = new Map<string, string>();
    for (const s of times) {
      map.set(parseSpecificDay(s).toString(), s);
    }
    return map;
  }, [times]);

  const renderCell = useCallback(
    (cell: DayCell) => {
      const inEvent = cell.isInEvent;

      if (!inEvent) {
        return (
          <div
            className={makeClass(monthStyles.date)}
            style={{
              opacity: 0.25,
              pointerEvents: "none",
              background: "transparent",
              borderColor: "transparent",
            }}
          >
            {cell.date.day}
          </div>
        );
      }

      const serialized = isoToSerialized.get(cell.iso) ?? cell.iso;
      let peopleHere = availabilities.find((a) => a.date === serialized)?.people ?? [];
      if (tempFocus) {
        peopleHere = peopleHere.filter((p) => p === tempFocus);
      }
      const color =
        palette[
          tempFocus && peopleHere.length
            ? Math.min(max, palette.length - 1)
            : Math.max(peopleHere.length - min, 0)
        ];

      const dimmed = focusCount !== undefined && focusCount !== peopleHere.length;
      const isHighlighted =
        highlight && (peopleHere.length === max || tempFocus) && peopleHere.length > 0 && !dimmed;

      const dateLabel = cell.date.toLocaleString(i18n.language, {
        weekday: "long",
        day: "numeric",
        month: "long",
      });

      return (
        <div
          className={makeClass(
            monthStyles.date,
            cell.isOtherMonth && monthStyles.otherMonth,
            cell.isToday && monthStyles.today,
            isHighlighted && styles.highlight,
          )}
          style={
            {
              backgroundColor: dimmed ? "transparent" : color.string,
              color: peopleHere.length > 0 && !dimmed ? "#fff" : undefined,
              "--highlight-color": color.highlight,
              cursor: "pointer",
            } as React.CSSProperties
          }
          onMouseEnter={(e) => {
            setTooltip({
              anchor: e.currentTarget,
              available: `${peopleHere.length} / ${filteredPeople.length} ${t("available")}`,
              date: dateLabel,
              people: peopleHere,
            });
          }}
          onMouseLeave={() => setTooltip(undefined)}
          onClick={() => {
            const clipboardMessage = `${t("group.clipboard_message", { date: dateLabel })}:\n${peopleHere.join(", ")}`;
            navigator.clipboard.writeText(clipboardMessage);
          }}
        >
          {cell.date.day}
        </div>
      );
    },
    [
      isoToSerialized,
      availabilities,
      tempFocus,
      palette,
      max,
      min,
      focusCount,
      highlight,
      filteredPeople,
      t,
      i18n.language,
    ],
  );

  return (
    <>
      <Content>
        <Legend
          min={min}
          max={max}
          total={filteredPeople.length}
          palette={palette}
          onSegmentFocus={setFocusCount}
        />

        <span className={styles.info}>{t("group.info1")}</span>

        {people.length > 1 && (
          <>
            <span className={styles.info}>{t("group.info2")}</span>
            <div className={styles.people}>
              {people.map((person) => (
                <button
                  type="button"
                  className={makeClass(
                    styles.person,
                    filteredPeople.includes(person.name) && styles.personSelected,
                  )}
                  key={person.name}
                  onClick={() => {
                    setTempFocus(undefined);
                    if (filteredPeople.includes(person.name)) {
                      setFilteredPeople(filteredPeople.filter((n) => n !== person.name));
                    } else {
                      setFilteredPeople([...filteredPeople, person.name]);
                    }
                  }}
                  onMouseOver={() => setTempFocus(person.name)}
                  onMouseOut={() => setTempFocus(undefined)}
                  title={relativeTimeFormat(
                    Temporal.Instant.fromEpochMilliseconds(person.created_at * 1000),
                    i18n.language,
                  )}
                >
                  {person.name}
                </button>
              ))}
            </div>
          </>
        )}
      </Content>

      <Content>
        <DayGrid eventDates={eventIsoSet} renderCell={renderCell} />

        {tooltip && (
          <div className={styles.tooltip} ref={refs.setFloating} style={floatingStyles}>
            <h3>{tooltip.available}</h3>
            <span>{tooltip.date}</span>
            {!!filteredPeople.length && (
              <div>
                {tooltip.people.map((person) => (
                  <span key={person}>{person}</span>
                ))}
                {filteredPeople
                  .filter((p) => !tooltip.people.includes(p))
                  .map((person) => (
                    <span key={person} data-disabled>
                      {person}
                    </span>
                  ))}
              </div>
            )}
          </div>
        )}
      </Content>
    </>
  );
};

export default AvailabilityViewerDays;
