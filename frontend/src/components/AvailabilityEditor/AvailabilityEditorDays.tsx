"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Temporal } from "@js-temporal/polyfill";

import Button from "/src/components/Button/Button";
import Content from "/src/components/Content/Content";
import DayGrid, { DayCell } from "/src/components/DayGrid/DayGrid";
import { usePalette } from "/src/hooks/usePalette";
import { useTranslation } from "/src/i18n/client";
import { makeClass, parseSpecificDay, serializeSpecificDay } from "/src/utils";

import styles from "./AvailabilityEditor.module.scss";
import monthStyles from "/src/components/CalendarField/components/Month/Month.module.scss";

interface AvailabilityEditorDaysProps {
  /** Event days as `DDMMYYYY` strings */
  times: string[];
  /** Currently selected days as `DDMMYYYY` strings */
  value: string[];
  onChange: (value: string[]) => void;
}

const AvailabilityEditorDays = ({ times, value = [], onChange }: AvailabilityEditorDaysProps) => {
  const { t } = useTranslation("event");

  const palette = usePalette(2);

  // Map ISO `YYYY-MM-DD` ↔ stored `DDMMYYYY` to bridge DayGrid and storage formats.
  const eventIsoSet = useMemo(
    () => new Set(times.map((s) => parseSpecificDay(s).toString())),
    [times],
  );
  const valueIsoSet = useMemo(
    () => new Set(value.map((s) => parseSpecificDay(s).toString())),
    [value],
  );

  // Drag-select state
  const selectingRef = useRef<Set<string>>(new Set());
  const [selecting, _setSelecting] = useState<Set<string>>(new Set());
  const setSelecting = useCallback((v: Set<string>) => {
    selectingRef.current = v;
    _setSelecting(v);
  }, []);
  const startCell = useRef<{ iso: string; date: Temporal.PlainDate } | undefined>(undefined);
  const mode = useRef<"add" | "remove">(undefined);

  const finishSelection = useCallback(() => {
    if (!mode.current) return;
    const selectedSerialized = [...selectingRef.current].map((iso) =>
      serializeSpecificDay(Temporal.PlainDate.from(iso)),
    );
    if (mode.current === "add") {
      onChange([...new Set([...value, ...selectedSerialized])]);
    } else {
      onChange(value.filter((s) => !selectedSerialized.includes(s)));
    }
    mode.current = undefined;
    setSelecting(new Set());
  }, [value, onChange, setSelecting]);

  // Bulk selection controls
  const selectAll = useCallback(() => onChange([...times]), [onChange, times]);
  const selectNone = useCallback(() => onChange([]), [onChange]);
  const selectInvert = useCallback(
    () => onChange(times.filter((t) => !value.includes(t))),
    [onChange, times, value],
  );

  // Keyboard shortcuts (mirror AvailabilityEditor)
  useEffect(() => {
    const handleKeydown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && (e.key === "a" || e.key === "i")) {
        e.preventDefault();
        if (e.shiftKey && e.key === "a") selectNone();
        else if (e.key === "a") selectAll();
        else selectInvert();
      }
    };
    document.addEventListener("keydown", handleKeydown);
    return () => document.removeEventListener("keydown", handleKeydown);
  }, [selectAll, selectNone, selectInvert]);

  const renderCell = useCallback(
    (cell: DayCell) => {
      const inEvent = cell.isInEvent;
      const isSelectingHere = selecting.has(cell.iso);
      const isCurrentlySelected = valueIsoSet.has(cell.iso);
      const isSelected =
        (!(mode.current === "remove" && isSelectingHere) && isCurrentlySelected) ||
        (mode.current === "add" && isSelectingHere);

      return (
        <button
          type="button"
          disabled={!inEvent}
          className={makeClass(
            monthStyles.date,
            cell.isOtherMonth && monthStyles.otherMonth,
            cell.isToday && monthStyles.today,
            isSelected && monthStyles.selected,
          )}
          style={{
            cursor: inEvent ? "pointer" : "not-allowed",
            opacity: inEvent ? 1 : 0.35,
            touchAction: "none",
            ...(inEvent && !isSelected ? { backgroundColor: palette[0].string } : null),
            ...(isSelected ? { backgroundColor: palette[1].string, color: "#fff" } : null),
          }}
          title={cell.date.toLocaleString(undefined, {
            day: "numeric",
            month: "long",
            year: "numeric",
          })}
          onPointerDown={(e) => {
            if (!inEvent) return;
            e.preventDefault();
            startCell.current = { iso: cell.iso, date: cell.date };
            mode.current = isCurrentlySelected ? "remove" : "add";
            setSelecting(new Set([cell.iso]));
            if (e.currentTarget.hasPointerCapture(e.pointerId)) {
              e.currentTarget.releasePointerCapture(e.pointerId);
            }
            document.addEventListener("pointerup", finishSelection, { once: true });
          }}
          onPointerEnter={() => {
            if (!mode.current || !startCell.current) return;
            const start = startCell.current.date;
            const end = cell.date;
            const [from, to] =
              Temporal.PlainDate.compare(start, end) <= 0 ? [start, end] : [end, start];
            const next = new Set<string>();
            let cur = from;
            while (Temporal.PlainDate.compare(cur, to) <= 0) {
              const iso = cur.toString();
              if (eventIsoSet.has(iso)) next.add(iso);
              cur = cur.add({ days: 1 });
            }
            setSelecting(next);
          }}
        >
          {cell.date.day}
        </button>
      );
    },
    [eventIsoSet, valueIsoSet, selecting, palette, finishSelection, setSelecting],
  );

  return (
    <>
      <Content isCentered>
        <div>{t("you.info")}</div>
        <div className={styles.selectionControls}>
          <Button isSmall onClick={selectAll} title="Ctrl + A (⌘ A)">
            {t("you.select_all")}
          </Button>
          <Button isSmall onClick={selectNone} title="Ctrl + Shift + A (⌘ ⇧ A)">
            {t("you.select_none")}
          </Button>
          <Button isSmall onClick={selectInvert} title="Ctrl + I (⌘ I)">
            {t("you.select_invert")}
          </Button>
        </div>
      </Content>

      <Content>
        <DayGrid eventDates={eventIsoSet} renderCell={renderCell} />
      </Content>
    </>
  );
};

export default AvailabilityEditorDays;
