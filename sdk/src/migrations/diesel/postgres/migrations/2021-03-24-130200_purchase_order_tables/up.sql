-- Copyright 2021 Cargill Incorporated
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.
-- -----------------------------------------------------------------------------

CREATE TABLE purchase_order (
    id INTEGER PRIMARY KEY,
    purchase_order_uid TEXT NOT NULL,
    buyer_org_id VARCHAR(256) NOT NULL,
    seller_org_id VARCHAR(256) NOT NULL,
    workflow_status TEXT NOT NULL,
    is_closed BOOLEAN NOT NULL,
    accepted_version_id TEXT,
    workflow_type TEXT NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

CREATE TABLE purchase_order_version (
    id INTEGER PRIMARY KEY,
    purchase_order_uid TEXT NOT NULL,
    version_id TEXT NOT NULL,
    is_draft BOOLEAN NOT NULL,
    current_revision_id BIGINT NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

CREATE TABLE purchase_order_version_revision (
    id INTEGER PRIMARY KEY,
    purchase_order_uid TEXT NOT NULL,
    version_id TEXT NOT NULL,
    revision_id BIGINT NOT NULL,
    order_xml_v3_4 TEXT NOT NULL,
    submitter TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

CREATE TABLE purchase_order_alternate_id (
    id INTEGER PRIMARY KEY,
    purchase_order_uid TEXT NOT NULL,
    org_id VARCHAR(256) NOT NULL,
    alternate_id_type TEXT NOT NULL,
    alternate_id TEXT NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);
