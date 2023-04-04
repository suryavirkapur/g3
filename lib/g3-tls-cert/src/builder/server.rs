/*
 * Copyright 2023 ByteDance and/or its affiliates.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use anyhow::{anyhow, Context};
use chrono::{DateTime, Datelike, Days, Utc};
use openssl::asn1::{Asn1Integer, Asn1Time};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use openssl::x509::extension::{
    ExtendedKeyUsage, KeyUsage, SubjectAlternativeName, SubjectKeyIdentifier,
};
use openssl::x509::{X509Builder, X509Extension, X509Name, X509Ref, X509};

use g3_types::net::Host;

pub struct ServerCertBuilder {
    pkey: PKey<Private>,
    serial: Asn1Integer,
    key_usage: X509Extension,
    ext_key_usage: X509Extension,
    not_before: Asn1Time,
    not_after: Asn1Time,
    digest: MessageDigest,
}

impl ServerCertBuilder {
    pub fn new() -> anyhow::Result<Self> {
        let pkey = super::pkey::new_ec()?;
        let serial = super::serial::random_16()?;

        let key_usage = KeyUsage::new()
            .critical()
            .digital_signature()
            .key_encipherment()
            .build()
            .map_err(|e| anyhow!("failed to build KeyUsage extension: {e}"))?;

        let ext_key_usage = ExtendedKeyUsage::new()
            .server_auth()
            .build()
            .map_err(|e| anyhow!("failed to build ExtendedKeyUsage extension: {e}"))?;

        let time_now = Utc::now();
        let time_before = time_now
            .checked_sub_days(Days::new(1))
            .ok_or(anyhow!("unable to get time before date"))?;
        let time_after = time_now
            .checked_add_days(Days::new(365))
            .ok_or(anyhow!("unable to get time after date"))?;
        let not_before =
            asn1_time_from_chrono(&time_before).context("failed to get NotBefore time")?;
        let not_after =
            asn1_time_from_chrono(&time_after).context("failed to set NotAfter time")?;

        Ok(ServerCertBuilder {
            pkey,
            serial,
            key_usage,
            ext_key_usage,
            not_before,
            not_after,
            digest: MessageDigest::sha256(),
        })
    }

    #[inline]
    pub fn pkey(&self) -> &PKey<Private> {
        &self.pkey
    }

    pub fn set_pkey(&mut self, pkey: PKey<Private>) {
        self.pkey = pkey;
    }

    pub fn refresh_pkey(&mut self) -> anyhow::Result<()> {
        self.pkey = super::pkey::new_ec()?;
        Ok(())
    }

    pub fn set_serial(&mut self, serial: Asn1Integer) {
        self.serial = serial;
    }

    pub fn refresh_serial(&mut self) -> anyhow::Result<()> {
        self.serial = super::serial::random_16()?;
        Ok(())
    }

    pub fn refresh_datetime(&mut self) -> anyhow::Result<()> {
        let time_now = Utc::now();
        let time_before = time_now
            .checked_sub_days(Days::new(1))
            .ok_or(anyhow!("unable to get time before date"))?;
        let time_after = time_now
            .checked_add_days(Days::new(365))
            .ok_or(anyhow!("unable to get time after date"))?;

        self.not_before =
            asn1_time_from_chrono(&time_before).context("failed to set NotBefore time")?;
        self.not_after =
            asn1_time_from_chrono(&time_after).context("failed to set NotAfter time")?;
        Ok(())
    }

    pub fn build_fake(
        &self,
        host: &Host,
        ca_cert: &X509Ref,
        ca_key: &PKey<Private>,
    ) -> anyhow::Result<X509> {
        let mut builder =
            X509Builder::new().map_err(|e| anyhow!("failed to create x509 builder {e}"))?;
        builder
            .set_pubkey(&self.pkey)
            .map_err(|e| anyhow!("failed to set pub key: {e}"))?;
        builder
            .set_serial_number(&self.serial)
            .map_err(|e| anyhow!("failed to set serial number: {e}"))?;

        let not_before = if ca_cert.not_before() > self.not_after {
            ca_cert.not_before()
        } else {
            &self.not_before
        };
        builder
            .set_not_before(not_before)
            .map_err(|e| anyhow!("failed to set NotBefore: {e}"))?;
        let not_after = if ca_cert.not_after() < self.not_after {
            ca_cert.not_after()
        } else {
            &self.not_after
        };
        builder
            .set_not_after(not_after)
            .map_err(|e| anyhow!("failed to set NotAfter: {e}"))?;

        builder
            .set_version(2)
            .map_err(|e| anyhow!("failed to set x509 version 3: {e}"))?;
        builder
            .append_extension2(&self.key_usage)
            .map_err(|e| anyhow!("failed to append KeyUsage extension: {e}"))?;
        builder
            .append_extension2(&self.ext_key_usage)
            .map_err(|e| anyhow!("failed to append ExtendedKeyUsage extension: {e}"))?;

        let mut san = SubjectAlternativeName::new();
        match host {
            Host::Domain(domain) => {
                let mut name = X509Name::builder().unwrap();
                name.append_entry_by_nid(Nid::COMMONNAME, domain).unwrap();
                let name = name.build();
                builder
                    .set_subject_name(&name)
                    .map_err(|e| anyhow!("failed to set subject name: {e}"))?;

                san.dns(domain);
            }
            Host::Ip(ip) => {
                let mut name = X509Name::builder().unwrap();
                let text = ip.to_string();
                name.append_entry_by_nid(Nid::COMMONNAME, &text).unwrap();
                let name = name.build();
                builder
                    .set_subject_name(&name)
                    .map_err(|e| anyhow!("failed to set subject name: {e}"))?;

                san.ip(&text);
            }
        }

        let v3_ctx = builder.x509v3_context(Some(ca_cert), None);
        let san = san
            .build(&v3_ctx)
            .map_err(|e| anyhow!("failed to build SubjectAlternativeName extension: {e}"))?;
        let ski = SubjectKeyIdentifier::new()
            .build(&v3_ctx)
            .map_err(|e| anyhow!("failed to build SubjectKeyIdentifier extension: {e} "))?;

        builder
            .append_extension(san)
            .map_err(|e| anyhow!("failed to append SubjectAlternativeName extension: {e}"))?;
        builder
            .append_extension(ski)
            .map_err(|e| anyhow!("failed to append SubjectKeyIdentifier extension: {e}"))?;

        builder
            .set_issuer_name(ca_cert.subject_name())
            .map_err(|e| anyhow!("failed to set issuer name: {e}"))?;
        builder
            .sign(ca_key, self.digest)
            .map_err(|e| anyhow!("failed to sign: {e}"))?;

        Ok(builder.build())
    }
}

fn asn1_time_from_chrono(datetime: &DateTime<Utc>) -> anyhow::Result<Asn1Time> {
    let lazy_fmt = if datetime.year() >= 2050 {
        datetime.format_with_items(g3_datetime::format::asn1::RFC5280_GENERALIZED.iter())
    } else {
        datetime.format_with_items(g3_datetime::format::asn1::RFC5280_UTC.iter())
    };
    Asn1Time::from_str(&format!("{lazy_fmt}")).map_err(|e| anyhow!("failed to get asn1 time: {e}"))
}