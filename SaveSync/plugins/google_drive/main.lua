function Info()
	return {
		name = "Google Drive",
		description = "Google Drive binding",
		author = "Zachareee",
		icon_url = "https://upload.wikimedia.org/wikipedia/commons/1/12/Google_Drive_icon_%282020%29.svg",
	}
end

--- @param credentials string Credential string produced on last initialisation of plugin
--- @return string?
function Validate(credentials, redirect_uri)
	local access, refresh = credentials:match("([^\n]+)\n([^\n]+)")
	local gdrive = require("gdrive")
	local gd, msg = gdrive.new({
		creds = {
			client_id = "487698375903-j8s33ij1pc335jc2pu6d2rb1bgrg2fqo.apps.googleusercontent.com",
			client_secret = "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",
		},
		redirect_uri,
		tokens = {
			access_token = access,
			refresh_token = refresh,
			token_type = "??????",
			expires = "????",
		},
	})
	return gd.tokenUpdated and nil or gd.acquireToken[1]
end

---@param url string
---@return string
function Extract_credentials(url)
	return url
end

--- Sync function
--- @param tag string Tagname for the folder
--- @param filename string Filename to upload
--- @param date_modified number Last modified date
--- @param zipbuffer string Buffer of the zipfile created
function Upload(tag, filename, date_modified, zipbuffer)
	local file = io.open("fake.zip", "wb")
	if file then
		file:write(zipbuffer)
		file:close()
	end
end

---@return {tag: string, filename: string, last_modified: {secs_since_epoch: number, nanos_since_epoch: string}, data: string?}[]?
function Read_cloud() end

---@return string
function Download(tag, filename)
	return "BINARYSTRING"
end

-- vim: ts=2 sw=2 et
