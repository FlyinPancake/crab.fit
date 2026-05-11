import { Suspense } from "react";
import { Metadata } from "next";
import { Temporal } from "@js-temporal/polyfill";

import Content from "/src/components/Content/Content";
import { getEvent } from "/src/app/actions";
import { useTranslation } from "/src/i18n/server";
import { relativeTimeFormat } from "/src/utils";

import EventAvailabilities from "./EventAvailabilities";
import EventShare from "./EventShare";
import styles from "./page.module.scss";

interface PageProps {
  params: Promise<{ id: string }>;
}

export const generateMetadata = async (props: PageProps): Promise<Metadata> => {
  const params = await props.params;
  const eventResponse = await getEvent(params.id);

  const { t } = await useTranslation("event");

  return {
    title: eventResponse.ok ? eventResponse.data.name : t("error.title"),
  };
};

const Page = async (props: PageProps) => {
  const params = await props.params;
  const eventResponse = await getEvent(params.id);

  if (!eventResponse.ok) return null;
  const event = eventResponse.data;

  const { t, i18n } = await useTranslation(["common", "event"]);

  return (
    <>
      <Suspense
        fallback={
          <Content>
            <h1 className={styles.name}>
              <span className={styles.bone} />
            </h1>
            <div className={styles.date}>
              <span className={styles.bone} />
            </div>
            <div className={styles.info}>
              <span className={styles.bone} style={{ width: "20em" }} />
            </div>
            <div className={styles.info}>
              <span className={styles.bone} style={{ width: "20em" }} />
            </div>
          </Content>
        }
      >
        <Content>
          <h1 className={styles.name}>{event.name}</h1>
          <span
            className={styles.date}
            title={Temporal.Instant.fromEpochMilliseconds(event.created_at * 1000).toLocaleString(
              i18n.language,
              { dateStyle: "long" },
            )}
          >
            {t("common:created", {
              date: relativeTimeFormat(
                Temporal.Instant.fromEpochMilliseconds(event.created_at * 1000),
                i18n.language,
              ),
            })}
          </span>

          <EventShare eventId={event.id} eventName={event.name} />
        </Content>
      </Suspense>

      <EventAvailabilities event={event} />
    </>
  );
};

export default Page;
