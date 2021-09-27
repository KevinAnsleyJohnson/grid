// Copyright 2021 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::convert::TryInto;
use std::time::{SystemTime, UNIX_EPOCH};

use grid_sdk::{
    client::purchase_order::{
        PurchaseOrder, PurchaseOrderClient, PurchaseOrderFilter, PurchaseOrderRevision,
        PurchaseOrderVersion,
    },
    protocol::purchase_order::payload::{
        Action, CreateVersionPayload, PurchaseOrderPayloadBuilder,
    },
    protos::IntoProto,
    purchase_order::addressing::GRID_PURCHASE_ORDER_NAMESPACE,
};

use chrono::{DateTime, Utc};
use cylinder::Signer;
use serde::Serialize;

use crate::error::CliError;
use crate::transaction::purchase_order_batch_builder;

pub fn do_create_version(
    client: &dyn PurchaseOrderClient,
    signer: Box<dyn Signer>,
    wait: u64,
    create_version: CreateVersionPayload,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

    let payload = PurchaseOrderPayloadBuilder::new()
        .with_action(Action::CreateVersion(create_version))
        .with_timestamp(timestamp)
        .build()
        .map_err(|err| CliError::UserError(format!("{}", err)))?;

    let batch_list = purchase_order_batch_builder(signer)
        .add_transaction(
            &payload.into_proto()?,
            &[GRID_PURCHASE_ORDER_NAMESPACE.to_string()],
            &[GRID_PURCHASE_ORDER_NAMESPACE.to_string()],
        )?
        .create_batch_list();

    client.post_batches(wait, &batch_list, service_id)?;
    Ok(())
}

pub fn do_fetch_revisions(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    version_id: &str,
    service_id: Option<&str>,
) -> Result<Vec<PurchaseOrderRevision>, CliError> {
    let revisions = client.list_purchase_order_revisions(
        po_uid.to_string(),
        version_id.to_string(),
        service_id,
    )?;

    Ok(revisions)
}

pub fn get_latest_revision_id(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    version_id: &str,
    service_id: Option<&str>,
) -> Result<i64, CliError> {
    let revisions = do_fetch_revisions(client, po_uid, version_id, service_id)?;

    let max = revisions.iter().max_by_key(|r| r.revision_id);

    if let Some(max) = max {
        Ok(max
            .revision_id
            .try_into()
            .map_err(|err| CliError::UserError(format!("{}", err)))?)
    } else {
        Ok(0)
    }
}

pub fn do_show_purchase_order(
    client: Box<dyn PurchaseOrderClient>,
    purchase_order_id: String,
    service_id: Option<String>,
    format: Option<&str>,
) -> Result<(), CliError> {
    let res = client.get_purchase_order(purchase_order_id, service_id.as_deref())?;
    match res {
        Some(purchase_order) => {
            print_formattable(PurchaseOrderCli::from(&purchase_order), format)?;
        }
        None => {
            println!("Purchase Order Not Found.");
        }
    }
    Ok(())
}

pub fn do_list_purchase_orders(
    client: Box<dyn PurchaseOrderClient>,
    filter: Option<PurchaseOrderFilter>,
    service_id: Option<String>,
    format: Option<&str>,
) -> Result<(), CliError> {
    let res = client.list_purchase_orders(filter, service_id.as_deref())?;
    let po_list = res
        .iter()
        .map(PurchaseOrderCli::from)
        .collect::<Vec<PurchaseOrderCli>>();
    print_formattable(PurchaseOrderCliVec(po_list), format)?;

    Ok(())
}
#[derive(Debug, Serialize)]
struct PurchaseOrderCli {
    org_id: String,
    purchase_order_uid: String,
    workflow_status: String,
    is_closed: bool,
    accepted_version_id: Option<String>,
    versions: Vec<PurchaseOrderVersionCli>,
    created_at: SystemTime,
}

impl From<&PurchaseOrder> for PurchaseOrderCli {
    fn from(d: &PurchaseOrder) -> Self {
        Self {
            org_id: d.org_id.to_string(),
            purchase_order_uid: d.purchase_order_uid.to_string(),
            workflow_status: d.workflow_status.to_string(),
            is_closed: d.is_closed,
            accepted_version_id: d.accepted_version_id.as_ref().map(String::from),
            versions: d
                .versions
                .iter()
                .map(PurchaseOrderVersionCli::from)
                .collect(),
            created_at: d.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
struct PurchaseOrderVersionCli {
    version_id: String,
    workflow_status: String,
    is_draft: bool,
    current_revision_id: u64,
    revisions: Vec<PurchaseOrderRevisionCli>,
}

impl From<&PurchaseOrderVersion> for PurchaseOrderVersionCli {
    fn from(d: &PurchaseOrderVersion) -> Self {
        Self {
            version_id: d.version_id.to_string(),
            workflow_status: d.workflow_status.to_string(),
            is_draft: d.is_draft,
            current_revision_id: d.current_revision_id,
            revisions: d
                .revisions
                .iter()
                .map(PurchaseOrderRevisionCli::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct PurchaseOrderRevisionCli {
    revision_id: u64,
    order_xml_v3_4: String,
    submitter: String,
    created_at: u64,
}

impl From<&PurchaseOrderRevision> for PurchaseOrderRevisionCli {
    fn from(d: &PurchaseOrderRevision) -> Self {
        Self {
            revision_id: d.revision_id,
            order_xml_v3_4: d.order_xml_v3_4.to_string(),
            submitter: d.submitter.to_string(),
            created_at: d.created_at,
        }
    }
}

#[derive(Serialize)]
pub struct PurchaseOrderCliVec(Vec<PurchaseOrderCli>);

impl std::fmt::Display for PurchaseOrderCliVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for purchase_order in &self.0 {
            write!(f, "\n\n{}", purchase_order)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for PurchaseOrderCli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Purchase Order {}:", &self.purchase_order_uid)?;
        write!(f, "\n\t{:18}{}", "Organization", &self.org_id)?;
        write!(f, "\n\t{:18}{}", "Workflow Status", &self.workflow_status)?;
        // Note: SystemTime format has not been agreed upon. This uses ISO 8601
        write!(
            f,
            "\n\t{:18}{:?}",
            "Created At",
            DateTime::<Utc>::from(self.created_at)
        )?;
        write!(f, "\n\t{:18}{}", "Closed", &self.is_closed)?;

        Ok(())
    }
}

fn print_formattable<T: std::fmt::Display + Serialize>(
    object: T,
    format: Option<&str>,
) -> Result<(), CliError> {
    match format {
        Some("json") => {
            let formatted = serde_json::to_string(&object)
                .map_err(|_| CliError::ActionError("Cannot Format Purchase Order".to_string()))?;
            println!("{}", formatted);
        }
        Some("yaml") => {
            let formatted = serde_yaml::to_string(&object)
                .map_err(|_| CliError::ActionError("Cannot Format Purchase Order".to_string()))?;
            println!("{}", formatted);
        }
        _ => println!("{}", object),
    }
    Ok(())
}
