"use client";

import { useEffect, useState } from "react";
import { Trans } from "react-i18next/TransWithoutContext";

import Copyable from "/src/components/Copyable/Copyable";
import { useTranslation } from "/src/i18n/client";
import { makeClass } from "/src/utils";

import styles from "./page.module.scss";

interface EventShareProps {
  eventId: string;
  eventName: string;
}

const getEventUrl = (eventId: string) => {
  if (typeof window === "undefined") return `https://crab.fit/${eventId}`;

  return `${window.location.origin}/${eventId}`;
};

const EventShare = ({ eventId, eventName }: EventShareProps) => {
  const { t, i18n } = useTranslation("event");
  const [eventUrl, setEventUrl] = useState(() => getEventUrl(eventId));

  useEffect(() => {
    setEventUrl(getEventUrl(eventId));
  }, [eventId]);

  return (
    <>
      <Copyable className={styles.info}>{eventUrl}</Copyable>
      <p className={makeClass(styles.info, styles.noPrint)}>
        <Trans i18nKey="nav.shareinfo" t={t} i18n={i18n}>
          _
          <a
            href={`mailto:?subject=${encodeURIComponent(t("nav.email_subject", { event_name: eventName }))}&body=${encodeURIComponent(`${t("nav.email_body")} ${eventUrl}`)}`}
          >
            _
          </a>
          _
        </Trans>
      </p>
    </>
  );
};

export default EventShare;
