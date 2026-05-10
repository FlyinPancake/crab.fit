import { getStats } from "/src/app/actions";
import { useTranslation } from "/src/i18n/server";

import styles from "./Stats.module.scss";

const Stats = async () => {
  const statsResponse = await getStats();

  const { t } = await useTranslation("home");

  return statsResponse.ok ? (
    <div className={styles.wrapper}>
      <div>
        <span className={styles.number}>
          {new Intl.NumberFormat().format(statsResponse.data.event_count)}
        </span>
        <span className={styles.label}>{t("about.events")}</span>
      </div>
      <div>
        <span className={styles.number}>
          {new Intl.NumberFormat().format(statsResponse.data.person_count)}
        </span>
        <span className={styles.label}>{t("about.availabilities")}</span>
      </div>
    </div>
  ) : null;
};

export default Stats;
